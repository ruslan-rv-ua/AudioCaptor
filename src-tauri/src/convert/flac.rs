//! FLAC files (`.flac`): libFLAC through the `libflac-sys` crate, with the
//! settings of decision-10. FLAC is lossless, so the check goes further than
//! for Opus and MP3: it decodes the whole file and compares every sample with
//! the WAV (decision-7).

use super::wav::WavPcm;
use super::{Codec, Control, ConvertError, Job};
use libflac_sys as flac;
use std::ffi::{c_char, c_void, CStr};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// libFLAC's own default: level 8 gives files 1.5-1.7 % smaller and takes
/// almost three times as long (TASK-11).
const COMPRESSION_LEVEL: u32 = 5;
/// Frames per FLAC block, and per read of the WAV.
const BLOCK_FRAMES: usize = 4096;
/// A seek point this often keeps seeking in a player quick.
const SEEK_POINT_SECONDS: u32 = 10;
/// The recorder writes 16-bit samples.
const BITS_PER_SAMPLE: u32 = 16;
/// Progress is reported once in this many blocks: about once a second of audio.
const BLOCKS_PER_REPORT: u64 = 12;

pub struct FlacCodec;

impl Codec for FlacCodec {
    fn extension(&self) -> &'static str {
        "flac"
    }

    /// An hour of audio took 8-9 s to encode and 3-4 s to check (TASK-11).
    fn encode_share(&self) -> f32 {
        0.7
    }

    fn encode(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError> {
        let mut wav = open_wav(job)?;
        let mut encoding = Encoding::start(job, File::create(output)?)?;
        let mut pcm = Vec::new();
        let mut wide: Vec<i32> = Vec::new();
        let mut done = 0u64;
        let mut blocks = 0u64;
        loop {
            control.check()?;
            let frames = wav.read(BLOCK_FRAMES, &mut pcm)?;
            if frames == 0 {
                break;
            }
            wide.clear();
            wide.extend(pcm.iter().map(|&sample| i32::from(sample)));
            encoding.write(&wide)?;
            done += frames as u64;
            if blocks.is_multiple_of(BLOCKS_PER_REPORT) {
                control.progress(done as f32 / job.frames as f32);
            }
            blocks += 1;
        }
        // An empty recording gives a valid file too: the headers and no sound.
        encoding.finish()
    }

    /// Decodes the whole file with libFLAC and compares every sample with the
    /// WAV. The WAV may be deleted only after this.
    fn verify(&self, job: &Job, output: &Path, control: &Control) -> Result<(), ConvertError> {
        control.check()?;
        let mut wav = open_wav(job)?;
        let channels = usize::from(job.channels);
        let mut pcm = Vec::new();
        let mut compared = 0u64;
        let mut blocks = 0u64;
        let info = decode_file(output, &mut |info, block| {
            control.check()?;
            if info.channels != u32::from(job.channels) {
                return Err(mismatch(info, job));
            }
            let frames = block.len() / channels;
            if wav.read(frames, &mut pcm)? != frames {
                return Err(ConvertError::failed(
                    "the FLAC file holds more sound than the recording",
                ));
            }
            let differs = block
                .iter()
                .zip(&pcm)
                .position(|(&decoded, &recorded)| decoded != i32::from(recorded));
            if let Some(sample) = differs {
                return Err(ConvertError::failed(format!(
                    "the FLAC file differs from the recording at frame {}",
                    compared + (sample / channels) as u64
                )));
            }
            compared += frames as u64;
            if blocks.is_multiple_of(BLOCKS_PER_REPORT) {
                control.progress(compared as f32 / job.frames as f32);
            }
            blocks += 1;
            Ok(())
        })?;

        if info.sample_rate != job.sample_rate
            || info.channels != u32::from(job.channels)
            || info.bits_per_sample != BITS_PER_SAMPLE
        {
            return Err(mismatch(&info, job));
        }
        // Both the sound in the file and the length a player reads from its
        // header must be the recording's.
        if compared != job.frames || info.frames != job.frames {
            return Err(ConvertError::failed(format!(
                "the FLAC file holds {compared} frames and says {}, the recording has {}",
                info.frames, job.frames
            )));
        }
        if !info.has_md5 {
            return Err(ConvertError::failed(
                "the FLAC file has no MD5 checksum of its sound",
            ));
        }
        Ok(())
    }
}

fn mismatch(info: &StreamInfo, job: &Job) -> ConvertError {
    ConvertError::failed(format!(
        "the FLAC file is {} Hz with {} channels and {} bits, the recording was {} Hz with {} and {BITS_PER_SAMPLE}",
        info.sample_rate, info.channels, info.bits_per_sample, job.sample_rate, job.channels
    ))
}

/// Opens the job's WAV and makes sure that it is the recording the job describes.
fn open_wav(job: &Job) -> Result<WavPcm, ConvertError> {
    let wav = WavPcm::open(&job.wav, job.frames)?;
    if wav.sample_rate() != job.sample_rate || wav.channels() != job.channels {
        return Err(ConvertError::failed(format!(
            "the WAV file is {} Hz with {} channels, the recording was {} Hz with {}",
            wav.sample_rate(),
            wav.channels(),
            job.sample_rate,
            job.channels
        )));
    }
    Ok(wav)
}

/// The text libFLAC has for the state of an encoder or a decoder.
fn state_text(state: *const c_char) -> String {
    if state.is_null() {
        return "unknown state".into();
    }
    // SAFETY: libFLAC returns one of its constant, NUL-terminated strings.
    unsafe { CStr::from_ptr(state) }
        .to_string_lossy()
        .into_owned()
}

/// The file an encoder writes to. libFLAC writes block by block and, at the
/// end, goes back to fill in the stream info (length, MD5) and the seek table.
struct Output {
    file: BufWriter<File>,
    /// The first write or seek that failed: libFLAC itself only learns that
    /// one did.
    error: Option<std::io::Error>,
}

unsafe extern "C" fn write_output(
    _: *const flac::FLAC__StreamEncoder,
    buffer: *const flac::FLAC__byte,
    bytes: usize,
    _samples: u32,
    _frame: u32,
    client: *mut c_void,
) -> flac::FLAC__StreamEncoderWriteStatus {
    if bytes == 0 {
        return flac::FLAC__STREAM_ENCODER_WRITE_STATUS_OK;
    }
    // SAFETY: `client` is the `Output` that `Encoding::start` handed to
    // libFLAC and that lives until the encoder is deleted; `buffer` holds
    // `bytes` bytes for the length of this call.
    let (output, data) = unsafe {
        (
            &mut *client.cast::<Output>(),
            std::slice::from_raw_parts(buffer, bytes),
        )
    };
    match output.file.write_all(data) {
        Ok(()) => flac::FLAC__STREAM_ENCODER_WRITE_STATUS_OK,
        Err(e) => {
            output.error.get_or_insert(e);
            flac::FLAC__STREAM_ENCODER_WRITE_STATUS_FATAL_ERROR
        }
    }
}

unsafe extern "C" fn seek_output(
    _: *const flac::FLAC__StreamEncoder,
    offset: flac::FLAC__uint64,
    client: *mut c_void,
) -> flac::FLAC__StreamEncoderSeekStatus {
    // SAFETY: as in `write_output`.
    let output = unsafe { &mut *client.cast::<Output>() };
    match output.file.seek(SeekFrom::Start(offset)) {
        Ok(_) => flac::FLAC__STREAM_ENCODER_SEEK_STATUS_OK,
        Err(e) => {
            output.error.get_or_insert(e);
            flac::FLAC__STREAM_ENCODER_SEEK_STATUS_ERROR
        }
    }
}

unsafe extern "C" fn tell_output(
    _: *const flac::FLAC__StreamEncoder,
    offset: *mut flac::FLAC__uint64,
    client: *mut c_void,
) -> flac::FLAC__StreamEncoderTellStatus {
    // SAFETY: as in `write_output`.
    let output = unsafe { &mut *client.cast::<Output>() };
    match output.file.stream_position() {
        Ok(position) => {
            // SAFETY: libFLAC passes a place for one number.
            unsafe { *offset = position };
            flac::FLAC__STREAM_ENCODER_TELL_STATUS_OK
        }
        Err(e) => {
            output.error.get_or_insert(e);
            flac::FLAC__STREAM_ENCODER_TELL_STATUS_ERROR
        }
    }
}

/// A libFLAC encoder at work, with the seek table it fills in and the file it
/// writes. Dropping it frees all three, finished or not.
struct Encoding {
    encoder: *mut flac::FLAC__StreamEncoder,
    /// A seek table with a point every `SEEK_POINT_SECONDS`; the encoder fills
    /// in where in the file each point is. It must outlive the encoder.
    table: *mut flac::FLAC__StreamMetadata,
    /// Owned here. libFLAC's callbacks reach it through the same pointer, so
    /// it is only touched through this pointer while the encoder lives.
    output: *mut Output,
    channels: usize,
}

impl Encoding {
    fn start(job: &Job, file: File) -> Result<Self, ConvertError> {
        // SAFETY: plain constructors; they return null when memory runs out.
        let (encoder, table) = unsafe {
            (
                flac::FLAC__stream_encoder_new(),
                flac::FLAC__metadata_object_new(flac::FLAC__METADATA_TYPE_SEEKTABLE),
            )
        };
        let output = Box::into_raw(Box::new(Output {
            file: BufWriter::with_capacity(1 << 16, file),
            error: None,
        }));
        // From here on `Drop` frees all three, whatever fails below.
        let encoding = Self {
            encoder,
            table,
            output,
            channels: usize::from(job.channels),
        };
        if encoder.is_null() || table.is_null() {
            return Err(ConvertError::failed("no memory for the FLAC encoder"));
        }

        // SAFETY: the encoder and the table are live, and the encoder is not
        // started yet, which is when it takes settings. It copies the list of
        // metadata blocks, so `blocks` may go once the call returns.
        let accepted = unsafe {
            let mut ok =
                flac::FLAC__metadata_object_seektable_template_append_spaced_points_by_samples(
                    table,
                    job.sample_rate.saturating_mul(SEEK_POINT_SECONDS),
                    job.frames,
                ) != 0;
            ok &= flac::FLAC__metadata_object_seektable_template_sort(table, 1) != 0;
            // The conversion checks the file itself, against the WAV.
            ok &= flac::FLAC__stream_encoder_set_verify(encoder, 0) != 0;
            ok &= flac::FLAC__stream_encoder_set_compression_level(encoder, COMPRESSION_LEVEL) != 0;
            ok &= flac::FLAC__stream_encoder_set_blocksize(encoder, BLOCK_FRAMES as u32) != 0;
            ok &= flac::FLAC__stream_encoder_set_channels(encoder, u32::from(job.channels)) != 0;
            ok &= flac::FLAC__stream_encoder_set_bits_per_sample(encoder, BITS_PER_SAMPLE) != 0;
            ok &= flac::FLAC__stream_encoder_set_sample_rate(encoder, job.sample_rate) != 0;
            ok &= flac::FLAC__stream_encoder_set_total_samples_estimate(encoder, job.frames) != 0;
            let mut blocks = [table];
            ok &= flac::FLAC__stream_encoder_set_metadata(encoder, blocks.as_mut_ptr(), 1) != 0;
            ok
        };
        if !accepted {
            return Err(ConvertError::failed(
                "the FLAC encoder does not take its settings",
            ));
        }

        // SAFETY: the callbacks match libFLAC's signatures, and `output` lives
        // until `Drop` has deleted the encoder. With seek and tell callbacks
        // libFLAC writes the final stream info and seek table itself.
        let status = unsafe {
            flac::FLAC__stream_encoder_init_stream(
                encoder,
                Some(write_output),
                Some(seek_output),
                Some(tell_output),
                None,
                output.cast(),
            )
        };
        if status != flac::FLAC__STREAM_ENCODER_INIT_STATUS_OK {
            return Err(encoding.failure(&format!("did not start (status {status})")));
        }
        Ok(encoding)
    }

    /// Encodes the next frames: interleaved samples, a whole number of frames.
    fn write(&mut self, interleaved: &[i32]) -> Result<(), ConvertError> {
        let frames = interleaved.len() / self.channels;
        // SAFETY: the encoder is started, and `interleaved` holds `frames`
        // frames of the channel count it was set to.
        let ok = unsafe {
            flac::FLAC__stream_encoder_process_interleaved(
                self.encoder,
                interleaved.as_ptr(),
                frames as u32,
            )
        };
        if ok == 0 {
            return Err(self.failure("stopped"));
        }
        Ok(())
    }

    /// Writes what is left, the final stream info and the seek table, and
    /// puts the file on the disk: the WAV may be deleted after the check.
    fn finish(self) -> Result<(), ConvertError> {
        // SAFETY: the encoder is started; this is its last call but `delete`.
        if unsafe { flac::FLAC__stream_encoder_finish(self.encoder) } == 0 {
            return Err(self.failure("did not finish the file"));
        }
        // SAFETY: `output` is the live box of `start`, and the finished
        // encoder calls back no more.
        let output = unsafe { &mut *self.output };
        if let Some(e) = output.error.take() {
            return Err(e.into());
        }
        output.file.flush()?;
        output.file.get_ref().sync_all()?;
        Ok(())
    }

    /// Why the encoder gave up: the error of the file if there is one, or
    /// else the encoder's own state.
    fn failure(&self, what: &str) -> ConvertError {
        // SAFETY: `output` is the live box of `start`, and no libFLAC call is
        // running that could call back into it.
        if let Some(e) = unsafe { (*self.output).error.take() } {
            return ConvertError::failed(format!("could not write the FLAC file: {e}"));
        }
        // SAFETY: a plain call on a live encoder.
        let state = state_text(unsafe {
            flac::FLAC__stream_encoder_get_resolved_state_string(self.encoder)
        });
        ConvertError::failed(format!("the FLAC encoder {what}: {state}"))
    }
}

impl Drop for Encoding {
    fn drop(&mut self) {
        // SAFETY: each pointer was made in `start` and is freed once, here.
        // Deleting an encoder that has not finished still writes through the
        // callbacks, so the encoder goes first and the output last.
        unsafe {
            if !self.encoder.is_null() {
                flac::FLAC__stream_encoder_delete(self.encoder);
            }
            if !self.table.is_null() {
                flac::FLAC__metadata_object_delete(self.table);
            }
            drop(Box::from_raw(self.output));
        }
    }
}

/// What the stream info block at the start of a FLAC file says.
#[derive(Debug, Clone, Copy, PartialEq)]
struct StreamInfo {
    sample_rate: u32,
    channels: u32,
    bits_per_sample: u32,
    /// Frames in the file: what a player takes for its length.
    frames: u64,
    /// Whether the block holds the MD5 checksum of the sound. libFLAC checks
    /// the decoded sound against it only when it does.
    has_md5: bool,
}

/// Receives each decoded block: the file's stream info and the samples of the
/// block, interleaved.
type Sink<'a> = dyn FnMut(&StreamInfo, &[i32]) -> Result<(), ConvertError> + 'a;

/// What the decoder's callbacks work on.
struct Reading<'a, 'b> {
    file: BufReader<File>,
    info: Option<StreamInfo>,
    interleaved: Vec<i32>,
    sink: &'a mut Sink<'b>,
    /// The first thing that went wrong; decoding stops there.
    error: Option<ConvertError>,
}

impl Reading<'_, '_> {
    fn fail(&mut self, error: ConvertError) {
        self.error.get_or_insert(error);
    }
}

unsafe extern "C" fn read_input(
    _: *const flac::FLAC__StreamDecoder,
    buffer: *mut flac::FLAC__byte,
    bytes: *mut usize,
    client: *mut c_void,
) -> flac::FLAC__StreamDecoderReadStatus {
    // SAFETY: `client` is the `Reading` of `decode_file`, which outlives the
    // decoder; `bytes` says how much room `buffer` has.
    let (reading, room) = unsafe {
        let reading = &mut *client.cast::<Reading>();
        if *bytes == 0 {
            return flac::FLAC__STREAM_DECODER_READ_STATUS_ABORT;
        }
        (reading, std::slice::from_raw_parts_mut(buffer, *bytes))
    };
    match reading.file.read(room) {
        Ok(read) => {
            // SAFETY: libFLAC reads back how many bytes it got.
            unsafe { *bytes = read };
            if read == 0 {
                flac::FLAC__STREAM_DECODER_READ_STATUS_END_OF_STREAM
            } else {
                flac::FLAC__STREAM_DECODER_READ_STATUS_CONTINUE
            }
        }
        Err(e) => {
            reading.fail(e.into());
            flac::FLAC__STREAM_DECODER_READ_STATUS_ABORT
        }
    }
}

unsafe extern "C" fn take_metadata(
    _: *const flac::FLAC__StreamDecoder,
    metadata: *const flac::FLAC__StreamMetadata,
    client: *mut c_void,
) {
    // SAFETY: `client` as in `read_input`; `metadata` is valid for this call.
    let (reading, block) = unsafe { (&mut *client.cast::<Reading>(), &*metadata) };
    if block.type_ == flac::FLAC__METADATA_TYPE_STREAMINFO {
        // SAFETY: the type says which member of the union is filled in.
        let info = unsafe { block.data.stream_info };
        reading.info = Some(StreamInfo {
            sample_rate: info.sample_rate,
            channels: info.channels,
            bits_per_sample: info.bits_per_sample,
            frames: info.total_samples,
            has_md5: info.md5sum != [0; 16],
        });
    }
}

unsafe extern "C" fn take_block(
    _: *const flac::FLAC__StreamDecoder,
    frame: *const flac::FLAC__Frame,
    buffer: *const *const flac::FLAC__int32,
    client: *mut c_void,
) -> flac::FLAC__StreamDecoderWriteStatus {
    // SAFETY: `client` as in `read_input`; `frame` is valid for this call.
    let (reading, header) = unsafe { (&mut *client.cast::<Reading>(), &(*frame).header) };
    let Some(info) = reading.info else {
        reading.fail(ConvertError::failed("the FLAC file has no stream info"));
        return flac::FLAC__STREAM_DECODER_WRITE_STATUS_ABORT;
    };
    // The files written here are mono or stereo with one format throughout.
    if header.channels != info.channels
        || header.sample_rate != info.sample_rate
        || header.bits_per_sample != info.bits_per_sample
        || !(1..=2).contains(&header.channels)
    {
        reading.fail(ConvertError::failed(
            "a block of the FLAC file does not match its stream info",
        ));
        return flac::FLAC__STREAM_DECODER_WRITE_STATUS_ABORT;
    }
    let frames = header.blocksize as usize;
    // SAFETY: `buffer` holds one array of `frames` samples for each channel.
    let channel = |index: usize| unsafe { std::slice::from_raw_parts(*buffer.add(index), frames) };
    reading.interleaved.clear();
    if header.channels == 1 {
        reading.interleaved.extend_from_slice(channel(0));
    } else {
        for (&left, &right) in channel(0).iter().zip(channel(1)) {
            reading.interleaved.push(left);
            reading.interleaved.push(right);
        }
    }
    match (reading.sink)(&info, &reading.interleaved) {
        Ok(()) => flac::FLAC__STREAM_DECODER_WRITE_STATUS_CONTINUE,
        Err(e) => {
            reading.fail(e);
            flac::FLAC__STREAM_DECODER_WRITE_STATUS_ABORT
        }
    }
}

unsafe extern "C" fn note_error(
    _: *const flac::FLAC__StreamDecoder,
    status: flac::FLAC__StreamDecoderErrorStatus,
    client: *mut c_void,
) {
    // SAFETY: `client` as in `read_input`.
    let reading = unsafe { &mut *client.cast::<Reading>() };
    let what = match status {
        flac::FLAC__STREAM_DECODER_ERROR_STATUS_LOST_SYNC => "it is cut or has stray bytes",
        flac::FLAC__STREAM_DECODER_ERROR_STATUS_BAD_HEADER => "a block has a bad header",
        flac::FLAC__STREAM_DECODER_ERROR_STATUS_FRAME_CRC_MISMATCH => "a block has a bad checksum",
        flac::FLAC__STREAM_DECODER_ERROR_STATUS_BAD_METADATA => "its metadata is damaged",
        _ => "the decoder cannot read it",
    };
    reading.fail(ConvertError::failed(format!(
        "the FLAC file is damaged: {what}"
    )));
}

/// Deletes a libFLAC decoder when it goes out of scope.
struct Decoder(*mut flac::FLAC__StreamDecoder);

impl Drop for Decoder {
    fn drop(&mut self) {
        // SAFETY: the pointer came from `FLAC__stream_decoder_new`, is not
        // null and is freed once, here.
        unsafe { flac::FLAC__stream_decoder_delete(self.0) };
    }
}

/// Decodes a whole FLAC file written by `FlacCodec` and fails if anything in
/// it is wrong: a block with a bad checksum, a cut, sound that does not match
/// the MD5 checksum in the stream info. Each decoded block goes to `sink`.
fn decode_file(path: &Path, sink: &mut Sink) -> Result<StreamInfo, ConvertError> {
    let mut reading = Reading {
        file: BufReader::with_capacity(1 << 16, File::open(path)?),
        info: None,
        interleaved: Vec::new(),
        sink,
        error: None,
    };
    // SAFETY: a plain constructor; null when memory runs out.
    let decoder = unsafe { flac::FLAC__stream_decoder_new() };
    if decoder.is_null() {
        return Err(ConvertError::failed("no memory for the FLAC decoder"));
    }
    let decoder = Decoder(decoder);
    let client: *mut Reading = &mut reading;

    // SAFETY: the decoder is live. The callbacks match libFLAC's signatures
    // and get `reading`, which outlives the decoder and is touched only
    // through `client` until the decoder is deleted. Without a seek callback
    // libFLAC needs no tell, length or end-of-file callback either: the file
    // is read front to back.
    let (decoded, state, md5_matches) = unsafe {
        flac::FLAC__stream_decoder_set_md5_checking(decoder.0, 1);
        let status = flac::FLAC__stream_decoder_init_stream(
            decoder.0,
            Some(read_input),
            None,
            None,
            None,
            None,
            Some(take_block),
            Some(take_metadata),
            Some(note_error),
            client.cast(),
        );
        if status != flac::FLAC__STREAM_DECODER_INIT_STATUS_OK {
            return Err(ConvertError::failed(format!(
                "the FLAC decoder did not start (status {status})"
            )));
        }
        let processed = flac::FLAC__stream_decoder_process_until_end_of_stream(decoder.0) != 0;
        let ended = flac::FLAC__stream_decoder_get_state(decoder.0)
            == flac::FLAC__STREAM_DECODER_END_OF_STREAM;
        let state = state_text(flac::FLAC__stream_decoder_get_resolved_state_string(
            decoder.0,
        ));
        // False when the decoded sound does not match the MD5 checksum.
        let md5_matches = flac::FLAC__stream_decoder_finish(decoder.0) != 0;
        (processed && ended, state, md5_matches)
    };
    drop(decoder);

    if let Some(error) = reading.error.take() {
        return Err(error);
    }
    if !decoded {
        return Err(ConvertError::failed(format!(
            "the FLAC file does not decode to its end: {state}"
        )));
    }
    if !md5_matches {
        return Err(ConvertError::failed(
            "the sound in the FLAC file does not match its MD5 checksum",
        ));
    }
    reading
        .info
        .ok_or_else(|| ConvertError::failed("the FLAC file has no stream info"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::types::OutputFormat;
    use crate::convert::test_util::{free_run, test_dir, tone_wav};
    use crate::convert::{convert_file, temp_path};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// Writes a 16-bit WAV that is hard on a lossless codec: the loudest and
    /// the quietest samples next to each other, a stretch of silence, a ramp,
    /// then noise, which does not compress at all. Returns the job for it.
    fn mixed_wav(path: &Path, sample_rate: u32, channels: u16, frames: u64) -> Job {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        let mut noise = 0x2545_F491_4F6C_DD1Du64;
        for frame in 0..frames {
            for channel in 0..u64::from(channels) {
                noise = noise
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                let sample = match frame {
                    // Left and right as far apart as 16 bits allow: their
                    // difference needs 17 bits.
                    0..=7 if (frame + channel) % 2 == 0 => i16::MAX,
                    0..=7 => i16::MIN,
                    8..=999 => 0,
                    1000..=2999 => (frame as i64 * 21 - 40_000 + channel as i64 * 5000) as i16,
                    _ => (noise >> 48) as i16,
                };
                writer.write_sample(sample).unwrap();
            }
        }
        writer.finalize().unwrap();
        Job {
            wav: path.to_path_buf(),
            frames,
            sample_rate,
            channels,
            format: OutputFormat::Flac,
        }
    }

    fn wav_samples(path: &Path) -> Vec<i32> {
        hound::WavReader::open(path)
            .unwrap()
            .samples::<i16>()
            .map(|sample| i32::from(sample.unwrap()))
            .collect()
    }

    /// Encodes `job` and checks it as the conversion does; returns the file.
    fn encode(job: &Job) -> PathBuf {
        let output = job.wav.with_extension("flac");
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        FlacCodec.encode(job, &output, &control).unwrap();
        FlacCodec.verify(job, &output, &control).unwrap();
        output
    }

    fn check(job: &Job, output: &Path) -> Result<(), ConvertError> {
        let (cancel, report) = free_run();
        FlacCodec.verify(job, output, &Control::new(cancel, report))
    }

    /// What a player gets from a FLAC file: its stream info and every sample.
    fn play(path: &Path) -> (StreamInfo, Vec<i32>) {
        let mut samples = Vec::new();
        let info = decode_file(path, &mut |_, block| {
            samples.extend_from_slice(block);
            Ok(())
        })
        .unwrap();
        (info, samples)
    }

    /// The metadata blocks at the start of a FLAC file: type and content.
    fn metadata_blocks(bytes: &[u8]) -> Vec<(u8, &[u8])> {
        assert_eq!(&bytes[..4], b"fLaC");
        let mut blocks = Vec::new();
        let mut at = 4;
        loop {
            let head = bytes[at];
            let len = u32::from_be_bytes([0, bytes[at + 1], bytes[at + 2], bytes[at + 3]]) as usize;
            blocks.push((head & 0x7F, &bytes[at + 4..at + 4 + len]));
            at += 4 + len;
            if head & 0x80 != 0 {
                return blocks;
            }
        }
    }

    #[test]
    fn every_profile_rate_gives_the_recording_sample_for_sample() {
        let dir = test_dir("flac-rates");
        for rate in [8_000u32, 16_000, 44_100, 48_000] {
            for channels in [1u16, 2] {
                let what = format!("{rate} Hz, {channels} channels");
                // 2.3 s: not a whole number of blocks.
                let frames = u64::from(rate) * 23 / 10;
                let job = mixed_wav(
                    &dir.join(format!("{rate}-{channels}.wav")),
                    rate,
                    channels,
                    frames,
                );
                let output = encode(&job);
                let (info, samples) = play(&output);

                assert_eq!(
                    info,
                    StreamInfo {
                        sample_rate: rate,
                        channels: u32::from(channels),
                        bits_per_sample: 16,
                        frames,
                        has_md5: true,
                    },
                    "{what}"
                );
                assert!(samples == wav_samples(&job.wav), "{what}: samples differ");
            }
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn empty_and_very_short_recordings_give_valid_files() {
        let dir = test_dir("flac-short");
        // Nothing, one frame, exactly one block, one frame more than a block.
        for frames in [0u64, 1, 4096, 4097] {
            let job = mixed_wav(&dir.join(format!("{frames}.wav")), 48_000, 2, frames);
            let (info, samples) = play(&encode(&job));
            assert_eq!(info.frames, frames);
            assert!(samples == wav_samples(&job.wav), "{frames} frames");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn file_has_the_length_an_md5_and_a_seek_point_every_ten_seconds() {
        let dir = test_dir("flac-headers");
        // 25 s at 8 kHz: 200 000 frames.
        let job = tone_wav(
            &dir.join("a.wav"),
            8_000,
            2,
            25.0,
            0.5,
            (440.0, 300.0),
            OutputFormat::Flac,
        );
        let output = encode(&job);
        let bytes = std::fs::read(&output).unwrap();
        // A tone compresses well; noise would not.
        assert!(bytes.len() as u64 * 2 < std::fs::metadata(&job.wav).unwrap().len());

        let blocks = metadata_blocks(&bytes);
        let (kind, info) = blocks[0];
        assert_eq!((kind, info.len()), (0, 34), "the stream info comes first");
        let block_sizes = (
            u16::from_be_bytes([info[0], info[1]]),
            u16::from_be_bytes([info[2], info[3]]),
        );
        assert_eq!(block_sizes, (4096, 4096));
        let packed = u64::from_be_bytes(info[10..18].try_into().unwrap());
        assert_eq!(packed >> 44, 8_000, "sample rate");
        assert_eq!(((packed >> 41) & 7) + 1, 2, "channels");
        assert_eq!(((packed >> 36) & 31) + 1, 16, "bits per sample");
        assert_eq!(packed & ((1 << 36) - 1), 200_000, "frames");
        assert_ne!(info[18..34], [0u8; 16], "MD5 of the sound");

        // A point for 0, 10 and 20 s. Each names the block that holds that
        // moment: blocks are 4096 frames, so they start at 0, 19 * 4096 and
        // 39 * 4096.
        let table = blocks
            .iter()
            .find(|(kind, _)| *kind == 3)
            .expect("a seek table")
            .1;
        let points: Vec<(u64, u64, u16)> = table
            .as_chunks::<18>()
            .0
            .iter()
            .map(|point| {
                (
                    u64::from_be_bytes(point[..8].try_into().unwrap()),
                    u64::from_be_bytes(point[8..16].try_into().unwrap()),
                    u16::from_be_bytes([point[16], point[17]]),
                )
            })
            .collect();
        let starts: Vec<u64> = points.iter().map(|point| point.0).collect();
        assert_eq!(starts, [0, 77_824, 159_744]);
        assert_eq!(points[0].1, 0, "the first block starts the sound");
        assert!(points[0].1 < points[1].1 && points[1].1 < points[2].1);
        assert!(points.iter().all(|point| point.2 == 4096));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_fails_when_one_sample_differs_from_the_wav() {
        let dir = test_dir("flac-sample");
        let job = mixed_wav(&dir.join("a.wav"), 48_000, 2, 30_000);
        let output = encode(&job);
        let wav = std::fs::read(&job.wav).unwrap();
        // The lowest bit of one sample, in the middle, at the start and at
        // the end of the recording. Frame 20 000 starts at byte 44 + 80 000.
        for (byte, frame) in [(44 + 80_000, 20_000), (44, 0), (wav.len() - 2, 29_999)] {
            let mut changed = wav.clone();
            changed[byte] ^= 1;
            std::fs::write(&job.wav, &changed).unwrap();
            let Err(ConvertError::Failed(reason)) = check(&job, &output) else {
                panic!("a changed sample in frame {frame} passed the check");
            };
            assert!(
                reason.ends_with(&format!("at frame {frame}")),
                "frame {frame}: {reason}"
            );
        }
        std::fs::write(&job.wav, &wav).unwrap();
        assert_eq!(check(&job, &output), Ok(()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_fails_when_the_file_is_not_as_long_as_the_recording() {
        let dir = test_dir("flac-length");
        let job = mixed_wav(&dir.join("a.wav"), 16_000, 2, 16_000);
        let output = encode(&job);
        // One frame more or less in the recording is already a mismatch.
        for frames in [job.frames + 1, job.frames - 1] {
            let other = Job {
                frames,
                ..job.clone()
            };
            let result = check(&other, &output);
            assert!(matches!(result, Err(ConvertError::Failed(_))), "{result:?}");
        }
        // The file of another recording: mono instead of stereo, another rate.
        let mono = mixed_wav(&dir.join("mono.wav"), 16_000, 1, 16_000);
        assert!(check(&mono, &output).is_err());
        let other_rate = mixed_wav(&dir.join("rate.wav"), 8_000, 2, 16_000);
        assert!(check(&other_rate, &output).is_err());
        assert_eq!(check(&job, &output), Ok(()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn check_fails_for_a_cut_or_damaged_file() {
        let dir = test_dir("flac-damaged");
        let job = tone_wav(
            &dir.join("a.wav"),
            48_000,
            2,
            3.0,
            0.0,
            (500.0, 700.0),
            OutputFormat::Flac,
        );
        let output = encode(&job);
        let good = std::fs::read(&output).unwrap();
        let fails = |bytes: &[u8], what: &str| {
            std::fs::write(&output, bytes).unwrap();
            let result = check(&job, &output);
            assert!(
                matches!(result, Err(ConvertError::Failed(_))),
                "{what}: {result:?}"
            );
        };

        fails(&good[..good.len() / 2], "cut in the middle");
        fails(&good[..good.len() - 1], "one byte short");
        let mut flipped = good.clone();
        flipped[good.len() / 2] ^= 0x10;
        fails(&flipped, "one flipped bit in the sound");
        // The MD5 checksum of the sound is the last 16 bytes of the stream
        // info: bytes 26 to 41 of the file.
        let mut other_md5 = good.clone();
        other_md5[30] ^= 0x01;
        fails(&other_md5, "another MD5 checksum");
        let mut no_md5 = good.clone();
        no_md5[26..42].fill(0);
        fails(&no_md5, "no MD5 checksum");
        fails(b"", "an empty file");
        fails(b"OggS this is not a FLAC file at all", "not a FLAC file");

        std::fs::write(&output, &good).unwrap();
        assert_eq!(check(&job, &output), Ok(()));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn encoding_fails_when_the_wav_is_not_the_recording() {
        let dir = test_dir("flac-mismatch");
        let job = mixed_wav(&dir.join("a.wav"), 48_000, 2, 24_000);
        let (cancel, report) = free_run();
        let control = Control::new(cancel, report);
        let output = dir.join("a.flac");
        // The queue says 44.1 kHz, the file is 48 kHz.
        let other_rate = Job {
            sample_rate: 44_100,
            ..job.clone()
        };
        assert!(FlacCodec.encode(&other_rate, &output, &control).is_err());
        // More frames counted than the file holds.
        let longer = Job {
            frames: job.frames + 10,
            ..job.clone()
        };
        assert!(FlacCodec.encode(&longer, &output, &control).is_err());
        let missing = Job {
            wav: dir.join("missing.wav"),
            ..job.clone()
        };
        assert!(FlacCodec.encode(&missing, &output, &control).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn conversion_leaves_only_the_flac_file() {
        let dir = test_dir("flac-convert");
        let job = mixed_wav(&dir.join("mix_2026-11-27_14-03-09.wav"), 44_100, 2, 50_000);
        let recorded = wav_samples(&job.wav);
        let (cancel, report) = free_run();

        let output = convert_file(&job, &FlacCodec, &Control::new(cancel, report)).unwrap();

        assert_eq!(output, dir.join("mix_2026-11-27_14-03-09.flac"));
        let left: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect();
        assert_eq!(left, std::slice::from_ref(&output));
        assert!(play(&output).1 == recorded);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn closing_the_app_stops_the_encoding_and_the_check() {
        let dir = test_dir("flac-cancel");
        let job = mixed_wav(&dir.join("mic.wav"), 48_000, 2, 240_000);
        let wav_before = std::fs::read(&job.wav).unwrap();
        let share = FlacCodec.encode_share();
        // The app starts closing during the encoding, and during the check.
        for (stage, from) in [("encoding", 0.0), ("check", share)] {
            let cancel = AtomicBool::new(false);
            let seen = std::sync::Mutex::new(Vec::new());
            let report = |done: f32| {
                seen.lock().unwrap().push(done);
                if done > from {
                    cancel.store(true, Ordering::Relaxed);
                }
            };

            let result = convert_file(&job, &FlacCodec, &Control::new(&cancel, &report));

            assert_eq!(result, Err(ConvertError::Cancelled), "{stage}");
            let seen = seen.into_inner().unwrap();
            // It stopped in that stage, well before the end.
            let last = *seen.last().unwrap();
            assert!(last > from && last < from + 0.2, "{stage}: {seen:?}");
            assert_eq!(std::fs::read(&job.wav).unwrap(), wav_before, "{stage}");
            assert!(!job.wav.with_extension("flac").exists(), "{stage}");
            assert!(
                !temp_path(&job.wav.with_extension("flac")).exists(),
                "{stage}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
