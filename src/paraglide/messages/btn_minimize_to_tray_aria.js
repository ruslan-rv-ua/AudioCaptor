/* eslint-disable */
import { getLocale, experimentalStaticLocale } from '../runtime.js';

/** @typedef {import('../runtime.js').LocalizedString} LocalizedString */

/** @typedef {{}} Btn_Minimize_To_Tray_AriaInputs */

const en_btn_minimize_to_tray_aria = /** @type {(inputs: Btn_Minimize_To_Tray_AriaInputs) => LocalizedString} */ () => {
	return /** @type {LocalizedString} */ (`Minimize to tray`)
};

const uk_btn_minimize_to_tray_aria = /** @type {(inputs: Btn_Minimize_To_Tray_AriaInputs) => LocalizedString} */ () => {
	return /** @type {LocalizedString} */ (`Згорнути в трей`)
};

/**
* | output |
* | --- |
* | "Minimize to tray" |
*
* @param {Btn_Minimize_To_Tray_AriaInputs} inputs
* @param {{ locale?: "en" | "uk" }} options
* @returns {LocalizedString}
*/
export const btn_minimize_to_tray_aria = /** @type {((inputs?: Btn_Minimize_To_Tray_AriaInputs, options?: { locale?: "en" | "uk" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Btn_Minimize_To_Tray_AriaInputs, { locale?: "en" | "uk" }, {}>} */ ((inputs = {}, options = {}) => {
	const locale = experimentalStaticLocale ?? options.locale ?? getLocale()
	if (locale === "en") return en_btn_minimize_to_tray_aria(inputs)
	return uk_btn_minimize_to_tray_aria(inputs)
});