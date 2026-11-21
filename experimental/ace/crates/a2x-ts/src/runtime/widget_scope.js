function a2x_widget_begin_frame() {
  _RT.budget_left = A2X_DEFAULT_BUDGET;
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
