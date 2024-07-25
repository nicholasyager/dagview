import './search.scss';
import { EventEmitter } from '../utilities/EventEmitter';

export type Selector = {
  parents: boolean;
  parents_depth?: number | undefined;
  value: string;
  children: boolean;
  children_depth?: number | undefined;
};

const RAW_SELECTOR_PATTERN =
  /^(?<parents>((?<parents_depth>(\d*))\+))?((?<value>.+?))?(?<children>(\+(?<children_depth>(\d*))))?$/;

export type SearchConfig = {
  rawSelector?: string;
};

export class SearchUI extends EventEmitter {
  constructor(state: SearchConfig = {}) {
    super();

    const container = document.createElement('div');
    container.classList.add('search-container');

    container.insertAdjacentHTML(
      'beforeend',
      `
      <form id="selectorForm">
            <input id="selectorSearch" placeholder="Node selector" value="${
              state.rawSelector || ''
            }"></input>
            <button id='selectorSubmit'>Select</button>
            </form>
        `
    );
    document.body.prepend(container);

    let selectorSubmitForm = document.getElementById('selectorForm');
    selectorSubmitForm.addEventListener('submit', (ev: SubmitEvent) => {
      ev.preventDefault();
      if (ev.target == null) {
        return;
      }
      let rawSelector = ev.target.firstElementChild.value;
      console.log(rawSelector);

      let match = RAW_SELECTOR_PATTERN.exec(rawSelector);
      if (!match) {
        throw new Error('Invalid selector spec');
      }

      const result: Record<string, string> = match.groups || {};
      console.log(result);
      // console.log(result);
      // result.childrens_parents = match?.[1];
      // result.parents = match?.[2] ? { depth: +match[2].match(/\d+/)[0], rest: match[2].replace(/^\d+\+/, '') } : undefined;
      // result.method = match?.[3];
      // result.value = match?.[5];
      // result.children_depth = match?.[7] ? +match[7] : undefined;

      let selector: Selector = {
        parents: !!result.parents,
        parents_depth: parseInt(result.parents_depth) || undefined,
        value: result.value as string,
        children: !!result.children,
        children_depth: parseInt(result.children_depth) || undefined,
      };

      if (this.listenerCount('search')) {
        this.emit('search', selector);
      }
    });
  }
}
