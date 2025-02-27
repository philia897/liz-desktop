import { getCurrentWindow } from "@tauri-apps/api/window"


document.addEventListener('DOMContentLoaded', async () => {
    const menuButton = document.getElementById("menu-button")!;
    const dropdownMenu = document.getElementById("dropdown-menu")!;
    const closeButton = document.getElementById("close-button")!;
    const commandsSection = document.getElementById("commands-section")!;
    const settingsSection = document.getElementById("settings-section")!;
    const tableBody = document.querySelector("#commands-table tbody")!;
    const editModal = document.getElementById("edit-modal")!;

    const contextMenu = document.createElement("div"); // For right click items

    let selectedRows: HTMLTableRowElement[] = [];
    let lastClickedRow: HTMLTableRowElement | null = null;

    const appWindow = getCurrentWindow();

    // Sample Data
    const commands = [
        { app: "App1", desc: "Test Desc", command: "Run", comment: "None", hit: 5 },
        { app: "App2", desc: "Another Desc", command: "Start", comment: "Check", hit: 10 },
        { app: "App3", desc: "Test Desc", command: "Run", comment: "None", hit: 5 },
        { app: "App4", desc: "Test Desc", command: "Run", comment: "None", hit: 5 },
    ];

    // Populate Table
    commands.forEach(cmd => {
        const row = document.createElement("tr");
        row.innerHTML = `<td>${cmd.app}</td><td>${cmd.desc}</td><td>${cmd.command}</td><td>${cmd.comment}</td><td>${cmd.hit}</td>`;
        tableBody.appendChild(row);
    });

    // Auto-select first item
    const firstRow = tableBody.querySelector("tr");
    if (firstRow) {
        firstRow.classList.add("selected");
        selectedRows = [firstRow];
        lastClickedRow = firstRow;
    }

    // Menu Toggle
    menuButton.addEventListener("click", (event) => {
        event.stopPropagation();
        dropdownMenu.classList.toggle("hidden");
    });

    const menuItems = dropdownMenu.querySelectorAll(".menu-item");
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
        editOption.classList.add("context-menu-item");
        editOption.textContent = "Edit";
        editOption.addEventListener("click", () => openEditModal(lastClickedRow));

        const deleteOption = document.createElement("div");
        deleteOption.textContent = "Delete";
        deleteOption.classList.add("context-menu-item");
        deleteOption.addEventListener("click", () => deleteSelectedRows());

        if (selectedRows.length === 1) {
            contextMenu.appendChild(editOption);
        }
        contextMenu.appendChild(deleteOption);

        // Position context menu
        contextMenu.style.top = `${event.clientY}px`;
        contextMenu.style.left = `${event.clientX}px`;
        contextMenu.classList.remove("hidden");
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
        (document.getElementById("edit-comment") as HTMLInputElement).value = cells[3].textContent || "";
        (document.getElementById("edit-hit") as HTMLInputElement).value = cells[4].textContent || "";
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
        if (!lastClickedRow) return;
        const cells = lastClickedRow.getElementsByTagName("td");
        cells[0].textContent = (document.getElementById("edit-app") as HTMLInputElement).value;
        cells[1].textContent = (document.getElementById("edit-desc") as HTMLInputElement).value;
        cells[2].textContent = (document.getElementById("edit-command") as HTMLInputElement).value;
        cells[3].textContent = (document.getElementById("edit-comment") as HTMLInputElement).value;
        cells[4].textContent = (document.getElementById("edit-hit") as HTMLInputElement).value;
        editModal.classList.add("hidden");
        commandsSection.classList.remove("hidden");
    });

    document.getElementById("cancel-edit")!.addEventListener("click", () => {
        editModal.classList.add("hidden");
        commandsSection.classList.remove("hidden");
    });
});