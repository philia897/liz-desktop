import { invoke } from '@tauri-apps/api/core';

/**
 * Fetch translations from Rust when needed.
 */
export async function getTranslations(keys: string[]): Promise<Record<string, string>> {
    try {
        return await invoke("get_translations", { keys });
    } catch (error) {
        console.error("Failed to load translations:", error);
        return {};
    }
}

/**
 * Get a translated string by key.
 */
export async function getTranslationItem(key: string): Promise<string> {
    try {
        return await invoke("get_translation", { key });
    } catch (error) {
        console.error("Failed to load translation:", error);
        return "";
    }
}

/**
 * Load language JSON file and update the UI accordingly.
 */
export async function loadLanguage(): Promise<void> {
    const elements = document.querySelectorAll("[data-i18n]");
    const keys = Array.from(elements).map((el) => el.getAttribute("data-i18n") || "");
    
    const translations = await getTranslations(keys);

    elements.forEach((el) => {
        const key = el.getAttribute("data-i18n");
        if (key && translations[key]) el.textContent = translations[key];
    });

    // Update placeholders
    const placeholders = document.querySelectorAll("[data-i18n-placeholder]");
    const placeholderKeys = Array.from(placeholders).map((el) => el.getAttribute("data-i18n-placeholder") || "");

    const placeholderTranslations = await getTranslations(placeholderKeys);
    
    placeholders.forEach((el) => {
        const key = el.getAttribute("data-i18n-placeholder");
        if (key && placeholderTranslations[key]) el.setAttribute("placeholder", placeholderTranslations[key]);
    });
}
