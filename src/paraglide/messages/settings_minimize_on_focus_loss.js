/* eslint-disable */
import { getLocale, experimentalStaticLocale } from '../runtime.js';

/** @typedef {import('../runtime.js').LocalizedString} LocalizedString */

/** @typedef {{}} Settings_Minimize_On_Focus_LossInputs */

const en_settings_minimize_on_focus_loss = /** @type {(inputs: Settings_Minimize_On_Focus_LossInputs) => LocalizedString} */ () => {
	return /** @type {LocalizedString} */ (`Minimize to tray when window loses focus`)
};

const uk_settings_minimize_on_focus_loss = /** @type {(inputs: Settings_Minimize_On_Focus_LossInputs) => LocalizedString} */ () => {
	return /** @type {LocalizedString} */ (`Згортати в трей при втраті фокуса`)
};

/**
* | output |
* | --- |
* | "Minimize to tray when window loses focus" |
*
* @param {Settings_Minimize_On_Focus_LossInputs} inputs
* @param {{ locale?: "en" | "uk" }} options
* @returns {LocalizedString}
*/
export const settings_minimize_on_focus_loss = /** @type {((inputs?: Settings_Minimize_On_Focus_LossInputs, options?: { locale?: "en" | "uk" }) => LocalizedString) & import('../runtime.js').MessageMetadata<Settings_Minimize_On_Focus_LossInputs, { locale?: "en" | "uk" }, {}>} */ ((inputs = {}, options = {}) => {
	const locale = experimentalStaticLocale ?? options.locale ?? getLocale()
	if (locale === "en") return en_settings_minimize_on_focus_loss(inputs)
	return uk_settings_minimize_on_focus_loss(inputs)
});