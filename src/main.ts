import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from "@tauri-apps/api/window";

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
  id: string;
  sc: string;
}

document.addEventListener('DOMContentLoaded', async () => {

  // let shortcuts: Shortcut[] = [];
  let shortcut_task: string = '';
  let selectedIndex = 0;
  let total_cnt = 0;

  const appWindow = getCurrentWindow();

  const shortcutListContainer = document.getElementById('shortcut-list') as HTMLUListElement;
  const searchBar = document.getElementById('search') as HTMLInputElement;
  const counter = document.getElementById("shortcut-counter") as HTMLSpanElement;

  function updateCounter() {
    const totalShortcuts = shortcutListContainer.children.length;
    counter.textContent = `${totalShortcuts} / ${total_cnt}`;
  }

  // Set the searchbar to be active in the beginning
  if (searchBar) {
    searchBar.focus();  // Focus on the search input when the window is active
  }

  // Fetch shortcuts from Rust via Tauri command
  async function fetchShortcuts(query: string) {
    const response: BlueBirdResponse = await invoke<BlueBirdResponse>('send_command', {
      cmd: { action: 'get_shortcuts', args: [query] },
    });
    if (response.code !== StateCode.OK) {
      alert(`Failed to retrieve shortcuts because ${response.results.join("; ")}`)
    }
    const shortcuts = response.results.map((content) => {
      // Parse the JSON string into a Shortcut
      return JSON.parse(content) as Shortcut;
    });
    if (!query) total_cnt = shortcuts.length;
    renderList(shortcuts, shortcutListContainer);
  }

  // refresh the shortcut list.
  listen('fetch-again', async () => {
    await fetchShortcuts("");
  });

  // Render the list of shortcuts
  function renderList(list: Shortcut[], listContainer: HTMLUListElement) {
    while (listContainer.firstChild) {
      listContainer.removeChild(listContainer.firstChild);
    } // Clear previous list

    const fragment = document.createDocumentFragment();

    list.forEach((item, index) => {
      const li = document.createElement('li');
      li.innerHTML = item.sc;
      li.id = item.id; // Store index for event delegation
      if (index === 0) li.classList.add('selected'); // First item selected by default
      fragment.appendChild(li);
    });

    listContainer.appendChild(fragment);
    selectedIndex = 0; // Reset selection on search input

    updateCounter();
  }

  function addClickListener(ul: HTMLUListElement) {
    // Event delegation for shortcut selection
    ul.addEventListener('click', (event) => {
      const target = (event.target as HTMLElement).closest('li');
      if (!target) return;

      const id = target.id;
      if (id) {
        executeShortcut(id);
      } else {
        console.error("Index is undefined");
      }
    });
  }

  addClickListener(shortcutListContainer);

  // Handle search input events
  const searchInput = document.getElementById('search') as HTMLInputElement;

  // Debounce function to limit the rate of search execution
  function debounce<T extends (...args: any[]) => void>(func: T, delay: number): T {
    let timer: ReturnType<typeof setTimeout>;
    return ((...args: Parameters<T>) => {
      clearTimeout(timer);
      timer = setTimeout(() => func(...args), delay);
    }) as T;
  }

  // Handle search input with debounce
  const debouncedSearch = debounce(async () => {
    const query = searchInput.value;
    await fetchShortcuts(query);
  }, 300);

  searchInput.addEventListener('input', debouncedSearch);

  // Helper function to scroll selected item into view
  function scrollSelectedItemIntoView(items: HTMLCollectionOf<HTMLLIElement>, index: number) {
    items[index].scrollIntoView({
      behavior: 'smooth',
      block: 'nearest',
    });
  }

  function _updateSelection(
    items: HTMLCollectionOf<HTMLLIElement>,
    newIndex: number
  ) {
    const oldIndex = selectedIndex
    if (items[oldIndex] && items[newIndex]) {
      items[oldIndex].classList.remove('selected');
      items[newIndex].classList.add('selected');
      selectedIndex = newIndex
    }
    scrollSelectedItemIntoView(items, newIndex);
  }

  // Keyboard navigation (up/down, esc and enter)
  document.addEventListener('keydown', (e) => {
    const ul = shortcutListContainer;
    const items = ul.getElementsByTagName('li');

    switch (e.key) {
      case 'ArrowDown':
        if (selectedIndex < items.length - 1) {
          _updateSelection(items, selectedIndex + 1);
        }
        e.preventDefault();
        break;

      case 'ArrowUp':
        if (selectedIndex > 0) {
          _updateSelection(items, selectedIndex - 1);
        }
        e.preventDefault();
        break;

      case 'Enter':
        if (selectedIndex >= 0 && selectedIndex < items.length) {
          const itemIdx = items[selectedIndex].id;
          if (itemIdx) {
            executeShortcut(itemIdx);
          } else {
            console.error('Index is undefined');
          }
        }
        break;

      case 'Escape':
        appWindow.hide(); // Hide the window
        break;
    }
  });

  // Execute the shortcut by set the shortcut_task, which will be executed when the window lost focus.
  async function executeShortcut(shortcut: string) {
    shortcut_task = shortcut;
    // Hide the window and start listening for focus changes
    await appWindow.hide();
    console.log("Window hidden. Waiting for focus loss...");
  }

  // When the window has lost focus, run the `shortcut_task` specified in `executeShortcut`
  appWindow.onFocusChanged(async () => {
    if (await appWindow.isFocused()) {
      if (searchBar) {
        searchBar.focus();  // Focus on the search input
      }
    } else {  // Ensure the window has truly lost focus and we have a valid `shortcut_task` to execute
      if (!shortcut_task) {
        // appWindow.hide()
        // resetView();
        // const ul = getActiveListContainer();
        // const items = ul.getElementsByTagName('li');
        // _updateSelection(items, 0);

        await appWindow.close();
        return
      }
      try {
        // Send the "execute" command to Rust with the shortcut task
        const response = await invoke<BlueBirdResponse>('send_command', {
          cmd: { action: 'execute', args: [shortcut_task] },
        });

        if (response.code !== StateCode.OK) {
          alert(`Failed to execute shortcut because ${response.results.join("; ")}`);
          console.log(`Failed to execute shortcut because ${response.results.join("; ")}`);
        }

        await appWindow.close();
        // Reset view and fetch updated shortcuts
        // resetView();
        // fetchShortcuts();  // Fetch the shortcuts again, as their ranking could change

      } catch (error) {
        console.error("Error executing shortcut:", error);
      } finally {
        // Reset the shortcut task after execution
        shortcut_task = '';
      }
    }
  });

  // Initialize by fetching shortcuts
  fetchShortcuts("");
});
