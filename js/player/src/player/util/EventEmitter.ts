/**
 * Minimal event emitter — a zero-dependency stand-in for `eventemitter3`.
 *
 * `@rlrml/player` ships with no runtime dependencies; the player follows suit so
 * consumers that compile the player's source (e.g. the stat-evaluation-player,
 * which symlinks this package) don't have to resolve an external emitter from
 * the player's own node_modules. The API is the small subset the adapter uses
 * (`on`/`once`/`off`/`emit`), shaped like eventemitter3's so call sites and
 * `extends EventEmitter` usage are unchanged.
 */
type Listener = (...args: any[]) => void;

export default class EventEmitter {
  private _listeners: Map<string, Set<Listener>> = new Map();

  on(event: string, fn: Listener): this {
    let set = this._listeners.get(event);
    if (!set) {
      set = new Set();
      this._listeners.set(event, set);
    }
    set.add(fn);
    return this;
  }

  once(event: string, fn: Listener): this {
    const wrapper: Listener = (...args) => {
      this.off(event, wrapper);
      fn(...args);
    };
    return this.on(event, wrapper);
  }

  off(event: string, fn?: Listener): this {
    const set = this._listeners.get(event);
    if (!set) return this;
    if (fn) set.delete(fn);
    else set.clear();
    if (set.size === 0) this._listeners.delete(event);
    return this;
  }

  removeListener(event: string, fn?: Listener): this {
    return this.off(event, fn);
  }

  removeAllListeners(event?: string): this {
    if (event) this._listeners.delete(event);
    else this._listeners.clear();
    return this;
  }

  emit(event: string, ...args: any[]): boolean {
    const set = this._listeners.get(event);
    if (!set || set.size === 0) return false;
    // Copy so listeners that remove themselves mid-dispatch don't skip others.
    for (const fn of [...set]) fn(...args);
    return true;
  }
}
