import './owner-filter.scss';
import { EventEmitter } from '../utilities/EventEmitter';

export class OwnerFilterUI extends EventEmitter {
  private enabledOwners: Set<string>;
  private checkboxes: Map<string, HTMLInputElement> = new Map();

  constructor(ownerColorMap: Map<string, string>) {
    super();

    // All owners start enabled. We use 'undefined' key for unassigned nodes.
    const ownerKeys = [...ownerColorMap.keys()];
    // Add a synthetic key for unassigned
    const allKeys = [...ownerKeys, 'undefined'];
    this.enabledOwners = new Set(allKeys);

    const container = document.createElement('div');
    container.classList.add('owner-filter-container');

    // Title
    const title = document.createElement('div');
    title.classList.add('owner-filter-title');
    title.textContent = 'Owner Groups';
    container.appendChild(title);

    // All / None actions
    const actions = document.createElement('div');
    actions.classList.add('owner-filter-actions');

    const allLink = document.createElement('a');
    allLink.textContent = 'All';
    allLink.addEventListener('click', () => this.setAll(true));

    const noneLink = document.createElement('a');
    noneLink.textContent = 'None';
    noneLink.addEventListener('click', () => this.setAll(false));

    actions.appendChild(allLink);
    actions.appendChild(noneLink);
    container.appendChild(actions);

    // Owner rows
    const list = document.createElement('div');
    list.classList.add('owner-filter-list');

    // Sorted owner entries followed by unassigned
    for (const ownerKey of allKeys) {
      const isUnassigned = ownerKey === 'undefined';
      const displayName = isUnassigned ? 'Unassigned' : ownerKey;
      const color = isUnassigned ? '#aaaaaa' : (ownerColorMap.get(ownerKey) ?? '#aaaaaa');

      const row = document.createElement('label');
      row.classList.add('owner-filter-row');

      const swatch = document.createElement('span');
      swatch.classList.add('owner-filter-swatch');
      swatch.style.backgroundColor = color;

      const checkbox = document.createElement('input');
      checkbox.type = 'checkbox';
      checkbox.checked = true;
      checkbox.classList.add('owner-filter-checkbox');
      checkbox.addEventListener('change', () => this.handleToggle(ownerKey, checkbox.checked));

      const label = document.createElement('span');
      label.classList.add('owner-filter-label');
      label.textContent = displayName;

      row.appendChild(swatch);
      row.appendChild(checkbox);
      row.appendChild(label);
      list.appendChild(row);

      this.checkboxes.set(ownerKey, checkbox);
    }

    container.appendChild(list);
    document.body.appendChild(container);
  }

  private handleToggle(ownerKey: string, checked: boolean) {
    if (checked) {
      this.enabledOwners.add(ownerKey);
    } else {
      this.enabledOwners.delete(ownerKey);
    }
    this.emitFilter();
  }

  private setAll(checked: boolean) {
    for (const [key, checkbox] of this.checkboxes) {
      checkbox.checked = checked;
      if (checked) {
        this.enabledOwners.add(key);
      } else {
        this.enabledOwners.delete(key);
      }
    }
    this.emitFilter();
  }

  private emitFilter() {
    this.emit('filter', { enabledOwners: new Set(this.enabledOwners) });
  }
}
