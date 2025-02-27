import { getCurrentWindow } from "@tauri-apps/api/window"

function searchTable(searchBox: HTMLInputElement, tableBody: HTMLTableSectionElement) {
    const searchText = searchBox.value.toLowerCase();  // Get the search input and convert it to lowercase
    const rows = tableBody.getElementsByTagName('tr');  // Get all rows in the table body
    
    // Loop through each row
    for (let row of rows) {
        const cells = row.getElementsByTagName('td');
        let matchFound = false;

        // Loop through each cell in the row
        for (let cell of cells) {
            const cellText = cell.textContent || cell.innerText;
            
            // If the search text is found in any cell, mark the row as a match
            if (cellText.toLowerCase().includes(searchText)) {
                matchFound = true;
                break;  // No need to continue checking other cells if a match is found
            }
        }

        row.style.display = matchFound ? '' : 'none';
    }
}

document.addEventListener('DOMContentLoaded', async () => {
    const menuButton = document.getElementById("menu-button")!;
    const dropdownMenu = document.getElementById("dropdown-menu")!;
    const closeButton = document.getElementById("close-button")!;
    const commandsSection = document.getElementById("commands-section")!;
    const settingsSection = document.getElementById("settings-section")!;
    const tableBody = document.querySelector("#commands-table tbody")!;
    const editModal = document.getElementById("edit-modal")!;

    let isCreatingNewCommand = false; // The state to sign if the application is creating new shortcut

    const searchBox = document.getElementById('search-box')!;

    const contextMenu = document.createElement("div"); // For right click items

    let selectedRows: HTMLTableRowElement[] = [];
    let lastClickedRow: HTMLTableRowElement | null = null;

    const appWindow = getCurrentWindow();

    // Sample Data
    const commands = [
        { app: "App1 cccccccccccccccc ccccccccccccccccc", desc: "Test Desc aaaaaaaaaaaaaaa aaaaaaaaaaaaaaaaaa", command: "Run", comment: "CMD Desc\n aaaaaaaaaaaaaaa aaaaaaaaaaaaaaaaaa", hit: 5 },
        { app: "App2", desc: "Another Desc", command: "Start", comment: "Check", hit: 10 },
        { app: "App3", desc: "Test Desc", command: "Run", comment: "None", hit: 5 },
        { app: "App4", desc: "Test Desc", command: "Run", comment: "None", hit: 5 },
    ];

    // Populate Table
    commands.forEach(cmd => {
        const row = document.createElement("tr");
        row.innerHTML = `<td>${cmd.app}</td><td title="${cmd.comment}">${cmd.desc}</td><td>${cmd.command}</td><td>${cmd.hit}</td>`;
        tableBody.appendChild(row);
    });

    // Add event listener for the 'keydown' event to check for Enter key
    if (searchBox instanceof HTMLInputElement && tableBody instanceof HTMLTableSectionElement)
        searchBox.addEventListener('keydown', (event) => {
            if (event.key === 'Enter') {  // Check if the Enter key was pressed
                event.preventDefault();  // Prevent form submission (if inside a form)
                searchTable(searchBox, tableBody);  // Trigger search when Enter is pressed
            }
        });

    // // Auto-select first item
    // const firstRow = tableBody.querySelector("tr");
    // if (firstRow) {
    //     firstRow.classList.add("selected");
    //     selectedRows = [firstRow];
    //     lastClickedRow = firstRow;
    // }

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
                selectedRows = rows.slice(Math.min(startIndex, endIndex), Math.max(startIndex, endIndex) + 1);
                selectedRows.forEach((r) => r.classList.add("selected"));
            } else if (event.ctrlKey) {
                // Ctrl + Click: Toggle selection
                row.classList.toggle("selected");
                if (selectedRows.includes(row)) {
                    selectedRows = selectedRows.filter((r) => r !== row);
                } else {
                    selectedRows.push(row);
                }
            } else {
                // Normal Click: Select only this row
                selectedRows.forEach((r) => r.classList.remove("selected"));
                row.classList.add("selected");
                selectedRows = [row];
            }
        }
        lastClickedRow = row;    
    });


    // Context Menu (Right-Click)
    contextMenu.classList.add("context-menu", "hidden");
    document.body.appendChild(contextMenu);

    tableBody.addEventListener("contextmenu", (event) => {
        event.preventDefault();

        const row = (event.target as HTMLElement).closest("tr")!;
        if (!row) return;

        if (!selectedRows.includes(row)) {
            selectedRows.forEach((r) => r.classList.remove("selected"));
            row.classList.add("selected");
            selectedRows = [row];
            lastClickedRow = row;
        }

        // Build context menu
        contextMenu.innerHTML = "";
        const editOption = document.createElement("div");
        editOption.classList.add("menu-item");
        editOption.textContent = "Edit";
        editOption.addEventListener("click", () => openEditModal(lastClickedRow));

        const deleteOption = document.createElement("div");
        deleteOption.textContent = "Delete Selected";
        deleteOption.classList.add("menu-item");
        deleteOption.addEventListener("click", () => deleteSelectedRows());

        const createOption = document.createElement("div");
        createOption.textContent = "New Item";
        createOption.classList.add("menu-item");
        createOption.addEventListener("click", () => createNewCommand());

        contextMenu.appendChild(createOption);
        if (selectedRows.length === 1) {
            contextMenu.appendChild(editOption);
            deleteOption.textContent = "Delete";
        }
        contextMenu.appendChild(deleteOption);

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
    function deleteSelectedRows() {
        selectedRows.forEach((row) => row.remove());
        selectedRows = [];
    }

    // Edit Save & Cancel Buttons
    document.getElementById("save-edit")!.addEventListener("click", () => {
        if (isCreatingNewCommand) {
            // Handle "Create New Command"
            const app = (document.getElementById("edit-app") as HTMLInputElement).value;
            const desc = (document.getElementById("edit-desc") as HTMLInputElement).value;
            const command = (document.getElementById("edit-command") as HTMLInputElement).value;
            const comment = (document.getElementById("edit-comment") as HTMLInputElement).value;
            const hit = parseInt((document.getElementById("edit-hit") as HTMLInputElement).value, 10);

            const newRow = document.createElement("tr");
            newRow.innerHTML = `
                <td>${app}</td>
                <td title="${comment}">${desc}</td>
                <td>${command}</td>
                <td>${hit}</td>
            `;

            // Append the new row to the table body
            tableBody.appendChild(newRow);

            // Set the newly created row as the last clicked row
            lastClickedRow = newRow;

            // Add the new row to the selected rows (if needed)
            selectedRows.push(newRow);

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
            const cells = lastClickedRow.getElementsByTagName("td");
            cells[0].textContent = (document.getElementById("edit-app") as HTMLInputElement).value;
            cells[1].textContent = (document.getElementById("edit-desc") as HTMLInputElement).value;
            cells[2].textContent = (document.getElementById("edit-command") as HTMLInputElement).value;
            cells[1].setAttribute('title', (document.getElementById("edit-command") as HTMLInputElement).value);
            cells[3].textContent = (document.getElementById("edit-hit") as HTMLInputElement).value;
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