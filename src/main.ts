import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from "@tauri-apps/api/window";
import Fuse from 'fuse.js';

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

let fuse: Fuse<Shortcut> | null = null;

// Initialize or Update Fuse
function updateFuse(shortcuts: Shortcut[]) {
  const fuseOptions = {
    keys: ['sc'], // Search within 'content'
    threshold: 0.4,    // Allow partial matches (lower is stricter)
    ignoreLocation: true, // Ignore match position for substring search
    includeMatches: true, // Include matched indices for highlighting
    useExtendedSearch: true, // Enable advanced search patterns
    minMatchCharLength: 2,   // Match single characters if needed
  };
  fuse = new Fuse(shortcuts, fuseOptions);
}

document.addEventListener('DOMContentLoaded', async () => {

  let shortcuts: Shortcut[] = [];
  let shortcut_task: string = '';
  let selectedIndex = 0;
  const appWindow = getCurrentWindow();

  const shortcutListContainer = document.getElementById('shortcut-list') as HTMLUListElement;
  const searchResultsContainer = document.getElementById('search-results') as HTMLUListElement;
  const searchBar = document.getElementById('search') as HTMLInputElement;

  // Set the searchbar to be active in the beginning
  if (searchBar) {
    searchBar.focus();  // Focus on the search input when the window is active
  }

  // Fetch shortcuts from Rust via Tauri command
  async function fetchShortcuts() {
    const response = await invoke<BlueBirdResponse>('send_command', {
      cmd: { action: 'get_shortcuts', args: [] },
    });
    if (response.code !== StateCode.OK) {
      alert(`Failed to retrieve shortcuts because ${response.results.join("; ")}`)
    }
    shortcuts = response.results.map((content) => {
      // Parse the JSON string into a Shortcut
      return JSON.parse(content) as Shortcut;
    });
    updateFuse(shortcuts);
    renderList(shortcuts, shortcutListContainer);
  }

  // refresh the shortcut list.
  listen('fetch-again', () => {
    fetchShortcuts();
  });

  async function resetView() {
    if (searchBar) {
      searchBar.value = ''; // Clear search input
      while (searchResultsContainer.firstChild) {
        searchResultsContainer.removeChild(searchResultsContainer.firstChild);
      }  // Clear previous search resulsts list
    }
    shortcutListContainer.style.display = 'block';
    shortcutListContainer.scrollTop = 0;
  }

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
  addClickListener(searchResultsContainer);

  // Perform Fuzzy Search
  function fuzzySearch(query: string): Shortcut[] {
    if (!query.trim() || !fuse) return shortcuts;
    return fuse.search(query).map(result => result.item);
  }

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
  const debouncedSearch = debounce(() => {
    const query = searchInput.value;
    if (query) {
      // If search query exists, hide the shortcut list and show search results
      shortcutListContainer.style.display = 'none';
      searchResultsContainer.style.display = 'block';

      // Perform the fuzzy search and render the search results
      const filtered = fuzzySearch(query);  // Assuming fuzzySearch is implemented
      renderList(filtered, searchResultsContainer);
    } else {
      // If no search query, hide search results and show the full list
      if (shortcutListContainer.style.display === 'none') {
        resetView()
        updateSelection(shortcutListContainer, 0);
      }
    }
  }, 300);

  searchInput.addEventListener('input', debouncedSearch);

  // Helper function to scroll selected item into view
  function scrollSelectedItemIntoView(items: HTMLCollectionOf<HTMLLIElement>, index: number) {
    items[index].scrollIntoView({
      behavior: 'smooth',
      block: 'nearest',
    });
  }

  // Helper function to update selection styling
  function updateSelection(ul: HTMLUListElement, newIndex: number) {
    const items = ul.getElementsByTagName('li');

    _updateSelection(items, newIndex);
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

  // Helper function to get active list container
  function getActiveListContainer(): HTMLUListElement {
    return shortcutListContainer.style.display === 'none'
      ? searchResultsContainer
      : shortcutListContainer;
  }

  // Keyboard navigation (up/down, esc and enter)
  document.addEventListener('keydown', (e) => {
    const ul = getActiveListContainer();
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
        resetView();
        _updateSelection(items, 0);
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
        appWindow.hide()
        resetView();
        const ul = getActiveListContainer();
        const items = ul.getElementsByTagName('li');
        _updateSelection(items, 0);
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
        // Reset view and fetch updated shortcuts
        resetView();
        fetchShortcuts();  // Fetch the shortcuts again, as their ranking could change

      } catch (error) {
        console.error("Error executing shortcut:", error);
      } finally {
        // Reset the shortcut task after execution
        shortcut_task = '';
      }
    }
  });

  // Initialize by fetching shortcuts
  fetchShortcuts();
});
