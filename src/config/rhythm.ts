import { invoke } from '@tauri-apps/api/core';
import { confirm, message } from '@tauri-apps/plugin-dialog'
import { relaunch } from '@tauri-apps/plugin-process';

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

function createSettingItem(setting: RhythmSetting): HTMLLabelElement {

    // Create the label element
    const label = document.createElement('label');
    label.id = setting.name
    label.innerHTML = formatString(setting.name); // Set the label text
    label.style.overflow = 'visible';
    label.classList.add("rhythm-setting");

    // Create the input element
    const input = document.createElement('input');
    input.type = "text"; // Set the input type to text
    input.id = `input-${setting.name}`; // Generate a dynamic ID based on the name
    input.value = setting.value; // Set the input value
    label.title = setting.hint; // Set the hover hint using the title attribute

    label.appendChild(input);

    return label;
}

function getSettingsJson(): Record<string, string | number> {
    const settingsObj: Record<string, string | number> = {}; // JSON object to store values

    // Select all labels with class "rhythm-setting"
    const labels = document.querySelectorAll<HTMLLabelElement>(".rhythm-setting");

    labels.forEach(label => {
        const input = label.querySelector("input"); // Find input inside label
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
    settings.forEach(setting => {
        const row = createSettingItem(setting)
        settingsContainer.appendChild(row);
    });

    // Add an event listener for the Save button
    saveButton.addEventListener('click', async () => {
        const confirmation = await confirm(
            `Are you sure you want to overwrite the settings?`,
            { title: 'Confirm to Overwrite Settings', kind: 'warning' }
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
                const confirmation = await confirm(`Update successfully.Some settings require restarting Liz to take effect. Restart Now?`, {
                    title: 'Restart Now?', kind: 'warning'
                })
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
        settings.forEach(setting => {
            const row = createSettingItem(setting)
            settingsContainer.appendChild(row);
        });
    });
}