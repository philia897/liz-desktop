import { getCurrentWindow } from "@tauri-apps/api/window"
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { confirm, message, open, save } from '@tauri-apps/plugin-dialog'
import { initialize_settings } from "./rhythm";
import { loadLanguage, getTranslations } from "../i18n"

const file_extensions = ['json', 'txt'];

enum StateCode {
    OK = "OK",
    FAIL = "FAIL",
    BUG = "BUG",
}

// Class for BlueBirdResponse
type BlueBirdResponse = {
    code: StateCode;
    results: string[];
}

type Shortcut = {
    id: string;         // Use string for large numbers (u128 in Rust)
    hit_number: number; // Can be a number since hit_number is an i64 in Rust
    shortcut: string;
    application: string;
    description: string;
    comment: string;
};

document.addEventListener('DOMContentLoaded', async () => {
    const menuButton = document.getElementById("menu-button")!;
    const dropdownMenu = document.getElementById("dropdown-menu")!;
    const closeButton = document.getElementById("close-button")!;
    const commandsSection = document.getElementById("commands-section")!;
    const settingsSection = document.getElementById("settings-section")!;
    const tableBody = document.querySelector("#commands-table tbody")!;
    const editModal = document.getElementById("edit-modal")!;
    let total_cnt = 0;

    loadLanguage()

    let isCreatingNewCommand = false; // The state to sign if the application is creating new shortcut

    const searchBox = document.getElementById('search-box') as HTMLInputElement;

    const contextMenu = document.createElement("div"); // For right click items

    let selectedRows = new Set<HTMLTableRowElement>();
    let lastClickedRow: HTMLTableRowElement | null = null;

    const appWindow = getCurrentWindow();

    const counter = document.getElementById("shortcut-counter") as HTMLSpanElement;

    function updateCounter() {
      const totalShortcuts = tableBody.children.length;
      counter.textContent = `${totalShortcuts} / ${total_cnt}`;
    }

    // Fetch shortcuts from Rust via Tauri command
    async function fetchShortcuts(query: string) {
        const response = await invoke<BlueBirdResponse>('send_command', {
            cmd: { action: 'get_shortcut_details', args: [query] },
        });
        if (response.code !== StateCode.OK) {
            alert(`Failed to retrieve shortcuts because ${response.results.join("; ")}`)
        }
        const shortcuts = response.results.map((content) => {
            // Parse the JSON string into a Shortcut
            return JSON.parse(content) as Shortcut;
        });
        if (!query) total_cnt = shortcuts.length;

        while (tableBody.firstChild) {
            tableBody.removeChild(tableBody.firstChild);
        } // Clear previous list

        // Populate Table
        shortcuts.forEach(cmd => {
            const row = document.createElement("tr");
            row.id = cmd.id
            row.innerHTML = `<td>${cmd.application}</td><td title="${cmd.comment}">${cmd.description}</td><td>${cmd.shortcut}</td><td>${cmd.hit_number}</td>`;
            tableBody.appendChild(row);
        });

        updateCounter();
    }

    // refresh the shortcut list.
    listen('fetch-again', async () => {
        await fetchShortcuts("");
    });

    // Check instance to avoid possible error of ts
    if (searchBox instanceof HTMLInputElement) {
        // Add event listener for the 'keydown' event to check for Enter key
        searchBox.addEventListener('keydown', (event) => {
            if (event.key === 'Enter') {
                event.preventDefault();  // Prevent form submission (if inside a form)

                // Clear selected rows
                selectedRows.clear();
                lastClickedRow = null;

                fetchShortcuts(searchBox.value);
            }
        });
    }

    fetchShortcuts("");

    // Menu Toggle
    menuButton.addEventListener("click", (event) => {
        event.stopPropagation();
        dropdownMenu.classList.toggle("hidden");
    });

    const menuItems = dropdownMenu.querySelectorAll(".dropdown-menu-item");
    // Handle menu item clicks
    menuItems.forEach((item) => {
        item.addEventListener("click", (event) => {
            const option = (event.target as HTMLElement).getAttribute("data-option");

            // Toggle sections based on the selected menu item
            if (option === "commands") {
                commandsSection.classList.remove("hidden");
                settingsSection.classList.add("hidden");
            } else if (option === "settings") {
                settingsSection.classList.remove("hidden");
                commandsSection.classList.add("hidden");

                initialize_settings()
            }

            // Hide dropdown after selection
            dropdownMenu.classList.add("hidden");
        });
    });

    // Close Button
    closeButton.addEventListener("click", async () => {
        await appWindow.close()
    });

    // Handle Table Click
    tableBody.addEventListener("click", (event) => {
        const row = (event.target as HTMLElement).closest("tr")!;
        if (!row) return;

        if (event instanceof MouseEvent) {
            if (event.shiftKey && lastClickedRow) {
                // Shift + Click: Select range
                const rows = Array.from(tableBody.querySelectorAll("tr"));
                const startIndex = rows.indexOf(lastClickedRow);
                const endIndex = rows.indexOf(row);
                const newSelection = rows.slice(Math.min(startIndex, endIndex), Math.max(startIndex, endIndex) + 1);
                // Add only new rows that are not already in the set
                newSelection.forEach((r) => {
                  if (!selectedRows.has(r)) {
                    selectedRows.add(r);
                    r.classList.add("selected");
                  }
                });
            } else if (event.ctrlKey) {
                // Ctrl + Click: Toggle selection
                row.classList.toggle("selected");
                if (selectedRows.has(row)) {
                    selectedRows.delete(row); // Remove if already selected
                } else {
                    selectedRows.add(row); // Add if not selected
                }
            } else {
                // Normal Click: Select only this row
                selectedRows.forEach((r) => r.classList.remove("selected"));
                row.classList.add("selected");
                selectedRows.clear();
                selectedRows.add(row);
            }
        }
        lastClickedRow = row;
    });


    // Context Menu (Right-Click)
    contextMenu.classList.add("context-menu", "hidden");
    document.body.appendChild(contextMenu);

    commandsSection.addEventListener("contextmenu", async (event) => {
        event.preventDefault();

        const row = (event.target as HTMLElement).closest("tr")!;
        if (!row) return;

        if (!selectedRows.has(row)) {
            selectedRows.forEach((r) => r.classList.remove("selected"));
            row.classList.add("selected");
            selectedRows.clear();
            selectedRows.add(row);
            lastClickedRow = row;
        }

        const translations = await getTranslations([
            "table_context_menu.delete_selected",
            "table_context_menu.delete",
            "table_context_menu.edit",
            "table_context_menu.new_item",
            "table_context_menu.export_selected",
            "table_context_menu.import_local"
        ])

        // Build context menu
        contextMenu.innerHTML = "";
        const editOption = document.createElement("div");
        editOption.classList.add("menu-item");
        editOption.textContent = translations["table_context_menu.edit"] || "Edit";
        editOption.addEventListener("click", () => openEditModal(lastClickedRow));

        const deleteOption = document.createElement("div");
        deleteOption.textContent = translations["table_context_menu.delete_selected"] || "Delete Selected";
        deleteOption.classList.add("menu-item");
        deleteOption.addEventListener("click", async () => {
            const translations = await getTranslations(["confirm_delete_message", "confirm_delete_title"]);
            const confirmation = await confirm(
                translations.confirm_delete_message.replace("{count}", selectedRows.size.toString()), 
                { title: translations.confirm_delete_title, kind: "warning" }
            );
            if (confirmation) {
                await deleteSelectedRows();
            }
        });

        const createOption = document.createElement("div");
        createOption.textContent = translations["table_context_menu.new_item"] || "New Item";
        createOption.classList.add("menu-item");
        createOption.addEventListener("click", () => createNewCommand());

        const exportOption = document.createElement("div");
        exportOption.textContent = translations["table_context_menu.export_selected"] || "Export Selected";
        exportOption.classList.add("menu-item");
        exportOption.addEventListener("click", () => exportSelectedRows());

        const importOption = document.createElement("div");
        importOption.textContent = translations["table_context_menu.import_local"] || "Import Local";
        importOption.classList.add("menu-item");
        importOption.addEventListener("click", () => importFromFileOrDir());

        contextMenu.appendChild(createOption);
        if (selectedRows.size === 1) {
            contextMenu.appendChild(editOption);
            deleteOption.textContent = translations["table_context_menu.delete"] || "Delete";
        }
        contextMenu.appendChild(deleteOption);
        contextMenu.appendChild(exportOption);
        contextMenu.appendChild(importOption);

        if (event instanceof MouseEvent) {
            // Position context menu
            contextMenu.style.top = `${event.clientY}px`;
            contextMenu.style.left = `${event.clientX}px`;
            contextMenu.classList.remove("hidden");
        }
    });

    // Close context menu on click outside
    document.addEventListener("click", () => {
        contextMenu.classList.add("hidden");
        dropdownMenu.classList.add("hidden");
    });

    // Edit Modal
    function openEditModal(row: HTMLTableRowElement | null) {
        if (!row) return;
        const cells = row.getElementsByTagName("td");
        (document.getElementById("edit-app") as HTMLInputElement).value = cells[0].textContent || "";
        (document.getElementById("edit-desc") as HTMLInputElement).value = cells[1].textContent || "";
        (document.getElementById("edit-command") as HTMLInputElement).value = cells[2].textContent || "";
        (document.getElementById("edit-comment") as HTMLInputElement).value = cells[1].getAttribute('title') || "";
        (document.getElementById("edit-hit") as HTMLInputElement).value = cells[3].textContent || "";
        commandsSection.classList.add("hidden");
        editModal.classList.remove("hidden");
    }

    // Function to open the modal for creating a new command
    function createNewCommand() {
        isCreatingNewCommand = true;  // Set the flag to indicate we're in creating mode

        // Clear all fields in the modal
        (document.getElementById("edit-app") as HTMLInputElement).value = "";
        (document.getElementById("edit-desc") as HTMLInputElement).value = "";
        (document.getElementById("edit-command") as HTMLInputElement).value = "";
        (document.getElementById("edit-comment") as HTMLInputElement).value = "";
        (document.getElementById("edit-hit") as HTMLInputElement).value = "0";

        // Show the edit modal and hide the commands section
        commandsSection.classList.add("hidden");
        editModal.classList.remove("hidden");
    }

    // Delete Function
    async function deleteSelectedRows() {
        {
            let idList = Array.from(selectedRows).map(row => row.id);
            const response = await invoke<BlueBirdResponse>('send_command', {
                cmd: { action: 'delete_shortcuts', args: idList },
            });
            if (response.code !== StateCode.OK) {
                await message(`Failed to delete shortcuts because ${response.results.join("; ")}`, {
                    title: 'Error information', kind: 'error'
                });
                return
            }
        }
        selectedRows.forEach((row) => row.remove());
        selectedRows.clear();
    }

    // Export Function
    async function exportSelectedRows() {
        const path = await save({
            filters: [
                {
                    name: 'Export to file',
                    extensions: file_extensions,
                },
            ],
        });
        let idList = Array.from(selectedRows).map(row => row.id);
        const response = await invoke<BlueBirdResponse>('send_command', {
            cmd: { action: 'export_shortcuts', args: [path].concat(idList) },
        });
        if (response.code !== StateCode.OK) {
            await message(`Failed to export shortcuts because ${response.results.join("; ")}`, {
                title: 'Error information', kind: 'error'
            });
            return
        }
    }

    // Import Function
    async function importFromFileOrDir() {
        const file_paths = await open({
            multiple: true,
            directory: false,
            filters: [{
                name: 'Import from file(s)',
                extensions: file_extensions,
            }],
        })
        if (!file_paths) {
            return
        }
        const response = await invoke<BlueBirdResponse>('send_command', {
            cmd: { action: 'import_shortcuts', args: file_paths },
        });
        if (response.code !== StateCode.OK) {
            await message(`Failed to import shortcuts: ${response.results.join("; ")}`, {
                title: 'Error information', kind: 'error'
            });
        }
    }

    // Edit Save & Cancel Buttons
    document.getElementById("save-edit")!.addEventListener("click", async () => {
        const app = (document.getElementById("edit-app") as HTMLInputElement).value;
        const desc = (document.getElementById("edit-desc") as HTMLInputElement).value;
        const command = (document.getElementById("edit-command") as HTMLInputElement).value;
        const comment = (document.getElementById("edit-comment") as HTMLInputElement).value;
        const hit_str = (document.getElementById("edit-hit") as HTMLInputElement).value;
        const hit = parseInt(hit_str, 10);

        if (isCreatingNewCommand) {
            // Handle "Create New Command"

            const newRow = document.createElement("tr");
            newRow.innerHTML = `
                <td>${app}</td>
                <td title="${comment}">${desc}</td>
                <td>${command}</td>
                <td>${hit}</td>
            `;

            // Ask for a new id from the backend
            const response = await invoke<BlueBirdResponse>('send_command', {
                cmd: { action: 'new_id', args: [] },
            });
            if (response.code !== StateCode.OK) {
                await message(`Failed to get ID because ${response.results.join("; ")}`, {
                    title: 'Error information', kind: 'error'
                });
                return
            }
            let new_id: string = response.results[0];
            newRow.id = new_id;


            {
                // Try to create a new shortcut
                let sc: Shortcut = {
                    id: newRow.id,
                    hit_number: hit,
                    shortcut: command,
                    application: app,
                    description: desc,
                    comment: comment
                }
                const response = await invoke<BlueBirdResponse>('send_command', {
                    cmd: { action: 'create_shortcuts', args: [JSON.stringify(sc)] },
                });
                if (response.code !== StateCode.OK) {
                    await message(`Failed to create shortcut because ${response.results.join("; ")}`, {
                        title: 'Error information', kind: 'error'
                    });
                    return
                }
            }

            // Append the new row to the table body
            tableBody.appendChild(newRow);

            // Set the newly created row as the last clicked row
            lastClickedRow = newRow;

            // Add the new row to the selected rows (if needed)
            selectedRows.add(newRow);

            // Scroll to the new row and highlight it
            newRow.scrollIntoView({ behavior: 'smooth', block: 'center' });
            newRow.classList.add("selected");

            // Hide the modal and show the commands section again
            editModal.classList.add("hidden");
            commandsSection.classList.remove("hidden");

            // Reset the creating mode state
            isCreatingNewCommand = false;
        } else {
            if (!lastClickedRow) return;

            {
                // Try to update a new shortcut
                let sc: Shortcut = {
                    id: lastClickedRow.id,
                    hit_number: hit,
                    shortcut: command,
                    application: app,
                    description: desc,
                    comment: comment
                }
                const response = await invoke<BlueBirdResponse>('send_command', {
                    cmd: { action: 'update_shortcuts', args: [JSON.stringify(sc)] },
                });
                if (response.code !== StateCode.OK) {
                    await message(`Failed to update shortcut ${response.results.join("; ")}`, {
                        title: 'Error information', kind: 'error'
                    });
                    return
                }
            }

            const cells = lastClickedRow.getElementsByTagName("td");
            cells[0].textContent = app;
            cells[1].textContent = desc;
            cells[2].textContent = command;
            cells[1].setAttribute('title', comment);
            cells[3].textContent = hit_str;
            editModal.classList.add("hidden");
            commandsSection.classList.remove("hidden");
        }

    });

    document.getElementById("cancel-edit")!.addEventListener("click", () => {
        editModal.classList.add("hidden");
        commandsSection.classList.remove("hidden");
        isCreatingNewCommand = false;
    });
});