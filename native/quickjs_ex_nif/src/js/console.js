// Console stub — silently discards output
// Override globalThis.console after start() to capture logs

(function() {
  var noop = function() {};
  globalThis.console = {
    log: noop, warn: noop, error: noop, info: noop, debug: noop, trace: noop,
    assert: noop, clear: noop, count: noop, countReset: noop,
    dir: noop, dirxml: noop, group: noop, groupCollapsed: noop, groupEnd: noop,
    table: noop, time: noop, timeEnd: noop, timeLog: noop
  };
})();
