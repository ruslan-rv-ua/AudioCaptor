use crate::audio::types::AudioDevice;
use wasapi::*;

/// Enumerates all active audio input devices (microphones).
pub fn list_input_devices() -> anyhow::Result<Vec<AudioDevice>> {
    let collection = DeviceCollection::new(&Direction::Capture)?;
    let count = collection.get_nbr_devices()?;
    let mut devices = Vec::new();

    for i in 0..count {
        let device = collection.get_device_at_index(i)?;
        let id = device.get_id()?;
        let name = device.get_friendlyname()?;
        devices.push(AudioDevice {
            id,
            name,
            is_input: true,
        });
    }

    Ok(devices)
}

/// Enumerates all active audio output devices (for loopback capture).
pub fn list_output_devices() -> anyhow::Result<Vec<AudioDevice>> {
    let collection = DeviceCollection::new(&Direction::Render)?;
    let count = collection.get_nbr_devices()?;
    let mut devices = Vec::new();

    for i in 0..count {
        let device = collection.get_device_at_index(i)?;
        let id = device.get_id()?;
        let name = device.get_friendlyname()?;
        devices.push(AudioDevice {
            id,
            name,
            is_input: false,
        });
    }

    Ok(devices)
}

/// Returns all audio devices (inputs + outputs).
pub fn list_all_devices() -> anyhow::Result<Vec<AudioDevice>> {
    let mut all = list_input_devices()?;
    all.extend(list_output_devices()?);
    Ok(all)
}

/// Find a device by its ID in the given direction.
pub fn get_device_by_id(id: &str, direction: &Direction) -> anyhow::Result<Device> {
    let collection = DeviceCollection::new(direction)?;
    let count = collection.get_nbr_devices()?;

    for i in 0..count {
        let device = collection.get_device_at_index(i)?;
        if device.get_id()? == id {
            return Ok(device);
        }
    }

    anyhow::bail!("Device not found: {}", id)
}
