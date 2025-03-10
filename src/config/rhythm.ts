import { invoke } from '@tauri-apps/api/core';
import { confirm, message } from '@tauri-apps/plugin-dialog'
import { relaunch } from '@tauri-apps/plugin-process';
import { getTranslations } from '../i18n';

enum StateCode {
    OK = "OK",
    FAIL = "FAIL",
    BUG = "BUG",
}

type BlueBirdResponse = {
    code: StateCode;
    results: string[];
}

type RhythmSetting = {
    name: string;
    value: string;
    hint: string;
}

function formatString(input: string): string {
    return input
        .split('_') // Split by underscores
        .map(word => word.charAt(0).toUpperCase() + word.slice(1)) // Capitalize each word
        .join(' '); // Join with spaces
}

async function createSettingItem(setting: RhythmSetting): Promise<HTMLLabelElement> {
    const name_key = `rhythm.${setting.name}`;
    const hint_key = `rhythm.${setting.name}.hint`;
    const translations = await getTranslations([name_key, hint_key]);

    // Create the label element
    const label = document.createElement('label');
    label.id = setting.name
    label.innerHTML = translations[name_key] || formatString(setting.name); // Set the label text
    label.style.overflow = 'visible';
    label.classList.add("rhythm-setting");

    let inputElement: HTMLInputElement | HTMLSelectElement;
    if (setting.name === "language") {
        // Create a select dropdown for language
        const select = document.createElement('select');
        select.id = `input-${setting.name}`;

        // Define supported languages
        const languages = [
            { code: "en", label: "English" },
            { code: "zh", label: "中文" }
        ];

        // Populate the select options
        languages.forEach(lang => {
            const option = document.createElement('option');
            option.value = lang.code;
            option.textContent = lang.label;
            if (setting.value === lang.code) {
                option.selected = true;
            }
            select.appendChild(option);
        });

        inputElement = select;
    } else {
        // Create the input element for other settings
        const input = document.createElement('input');
        input.type = "text"; // Set the input type to text
        input.id = `input-${setting.name}`; // Generate a dynamic ID based on the name
        input.value = setting.value; // Set the input value
        inputElement = input;
    }
    label.title = translations[hint_key] || setting.hint; // Set the hover hint using the title attribute

    label.appendChild(inputElement);

    return label;
}

function getSettingsJson(): Record<string, string | number> {
    const settingsObj: Record<string, string | number> = {}; // JSON object to store values

    // Select all labels with class "rhythm-setting"
    const labels = document.querySelectorAll<HTMLLabelElement>(".rhythm-setting");

    labels.forEach(label => {
        const input = label.querySelector("input") || label.querySelector("select"); // Find input inside label
        if (input) {
            const value = input.value.trim(); // Get and trim the value
            if (value) {
                // Convert to number if it's a valid numeric value, otherwise keep as string
                settingsObj[label.id] = isNaN(Number(value)) ? value : Number(value);
            }  // If value is missing, the backend will reset it to default value.
        }
    });

    return settingsObj;
}

export async function initialize_settings() {

    const settingsContainer = document.getElementById('settings-container')!;
    const saveButton = document.getElementById('save-rhytm')!;
    const resetButton = document.getElementById('reset-rhytm')!;

    const response = await invoke<BlueBirdResponse>('send_command', {
        cmd: { action: 'info', args: [] },
    });
    if (response.code !== StateCode.OK) {
        await message(`Failed to retrieve rhythm info because ${response.results.join("; ")}`, {
            title: 'Error information', kind: 'error'
        });
        return
    }
    let settings: RhythmSetting[] = [];
    settings = response.results.map((content) => {
        // Parse the JSON string into a Shortcut
        return JSON.parse(content) as RhythmSetting;
    });

    while (settingsContainer.firstChild) {
        settingsContainer.removeChild(settingsContainer.firstChild);
    } // Clear previous list

    // Populate Table
    settings.forEach(async setting => {
        const row = await createSettingItem(setting)
        settingsContainer.appendChild(row);
    });

    // Add an event listener for the Save button
    saveButton.addEventListener('click', async () => {
        const translations = await getTranslations(["confirm_overwrite_settings", "confirm_overwrite_settings_title"]);
        const confirmation = await confirm(
            translations.confirm_overwrite_settings || "Are you sure you want to overwrite the settings?", 
            { title: translations.confirm_overwrite_settings_title || "Confirm to Overwrite Settings", kind: "warning" }
        );
        if (confirmation) {
            const settingsJson = getSettingsJson();
            const response = await invoke<BlueBirdResponse>('send_command', {
                cmd: { action: 'update_rhythm', args: [JSON.stringify(settingsJson)] },
            });
            if (response.code !== StateCode.OK) {
                await message(`Failed to update rhythm because ${response.results.join("; ")}`, {
                    title: 'Error information', kind: 'error'
                });
                return
            } else {
                const translations = await getTranslations(["ask_to_restart", "ask_to_restart_title"]);
                const confirmation = await confirm(
                    translations.ask_to_restart || "Update successfully. Some settings require restarting Liz to take effect. Restart Now?", 
                    { title: translations.ask_to_restart_title || "Restart Now?", kind: 'warning'}
                )
                if (confirmation) {
                    await relaunch();
                }
            }

        }
    });

    // Add an event listener for the Reset button
    resetButton.addEventListener('click', () => {
        while (settingsContainer.firstChild) {
            settingsContainer.removeChild(settingsContainer.firstChild);
        } // Clear previous list
        settings.forEach(async setting => {
            const row = await createSettingItem(setting)
            settingsContainer.appendChild(row);
        });
    });
}