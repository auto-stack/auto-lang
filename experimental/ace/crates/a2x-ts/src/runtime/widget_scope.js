const _widgetModuleConstants = new Map();

function a2x_widget_global_init(symbol, value, text) {
  if (_widgetModuleConstants.has(symbol)) {
    _trap('Internal', `module constant ${symbol} was initialized twice`);
  }
  // Snapshot text as bytes owned by the factory. The per-frame text ledger
  // can release the initializer's temporary without invalidating the cell.
  _widgetModuleConstants.set(symbol, text ? a2x_str_to_borrowed(value).slice() : value);
  return 0;
}

function a2x_widget_global_get(symbol, owned) {
  if (!_widgetModuleConstants.has(symbol)) {
    _trap('Internal', `module constant ${symbol} was read before initialization`);
  }
  const value = _widgetModuleConstants.get(symbol);
  return owned ? a2x_str_clone(value) : value;
}

function a2x_widget_begin_frame(budget = A2X_DEFAULT_BUDGET) {
  _RT.budget_left = budget;
}

function a2x_widget_work_budget() {
  return _RT.budget_left;
}

function a2x_widget_step(held, run) {
  const first = _RT.next_id;
  const step = run();
  if (!step.done) {
    for (let id = first; id < _RT.next_id; id++) {
      const text = _RT.live_strs.get(id);
      if (text !== undefined) held.add(text);
    }
  }
  return step;
}

function a2x_widget_copy_state(value) {
  const copies = new Map();
  const pending = [];
  function copy(item) {
    if (item === null || typeof item !== 'object' || ArrayBuffer.isView(item)) return item;
    if (copies.has(item)) return copies.get(item);
    a2x_bill(1);
    if (item instanceof A2xOwnedStr) {
      const text = a2x_str_clone(item);
      copies.set(item, text);
      return text;
    }
    const target = Array.isArray(item) ? new Array(item.length) : Object.create(Object.getPrototypeOf(item));
    copies.set(item, target);
    pending.push([item, target]);
    return target;
  }
  const root = copy(value);
  while (pending.length !== 0) {
    const [source, target] = pending.pop();
    for (const [key, item] of Object.entries(source)) {
      Object.defineProperty(target, key, { value: copy(item), enumerable: true, writable: true, configurable: true });
    }
  }
  return root;
}

function a2x_widget_keep_models(models) {
  // Checked model records and collections can share values. Visit each object
  // once, and release only text owned by this factory's ledger.
  const retained = new Set();
  const pending = Array.from(models);
  while (pending.length !== 0) {
    const value = pending.pop();
    if (value === null || typeof value !== 'object' || retained.has(value)) continue;
    retained.add(value);
    if (value instanceof A2xOwnedStr || ArrayBuffer.isView(value)) continue;
    for (const child of Object.values(value)) pending.push(child);
  }
  for (const owned of _RT.live_strs.values()) {
    if (!retained.has(owned)) a2x_str_free(owned);
  }
  // The embedded program uses the ledger to enforce ownership, without
  // retaining a trace of every text released by prior view updates.
  _RT.freed_strs.length = 0;
}
