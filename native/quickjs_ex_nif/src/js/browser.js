// Browser environment stubs for SSR and bundled npm packages
// Provides minimal implementations of browser/Node globals

(function() {
  globalThis.window = globalThis;
  globalThis.self = globalThis;

  globalThis.navigator = { userAgent: '', language: 'en', platform: '' };

  globalThis.location = {
    href: '', protocol: 'https:', host: '', hostname: '', port: '',
    pathname: '/', search: '', hash: '', origin: ''
  };

  globalThis.document = {
    createElement: function(tag) {
      return {
        tagName: tag.toUpperCase(),
        style: {},
        setAttribute: function() {},
        getAttribute: function() { return null; },
        appendChild: function() {},
        removeChild: function() {},
        addEventListener: function() {},
        removeEventListener: function() {},
        classList: { add: function() {}, remove: function() {}, toggle: function() {}, contains: function() { return false; } },
        children: [], childNodes: [], innerHTML: '', textContent: ''
      };
    },
    createTextNode: function(text) { return { nodeType: 3, textContent: text }; },
    createDocumentFragment: function() { return { appendChild: function() {}, childNodes: [] }; },
    createComment: function(text) { return { nodeType: 8, textContent: text }; },
    querySelector: function() { return null; },
    querySelectorAll: function() { return []; },
    getElementById: function() { return null; },
    getElementsByClassName: function() { return []; },
    getElementsByTagName: function() { return []; },
    addEventListener: function() {},
    removeEventListener: function() {},
    head: { appendChild: function() {}, removeChild: function() {} },
    body: { appendChild: function() {}, removeChild: function() {} },
    documentElement: { style: {} }
  };

  globalThis.process = { env: { NODE_ENV: 'production' }, version: '' };

  var noopStorage = {
    getItem: function() { return null; },
    setItem: function() {},
    removeItem: function() {},
    clear: function() {},
    key: function() { return null; },
    length: 0
  };
  globalThis.localStorage = noopStorage;
  globalThis.sessionStorage = noopStorage;

  globalThis.Event = globalThis.Event || function Event(type, options) {
    this.type = type;
    this.bubbles = !!(options && options.bubbles);
    this.cancelable = !!(options && options.cancelable);
    this.defaultPrevented = false;
    this.preventDefault = function() { this.defaultPrevented = true; };
    this.stopPropagation = function() {};
    this.stopImmediatePropagation = function() {};
  };

  globalThis.CustomEvent = globalThis.CustomEvent || function CustomEvent(type, options) {
    Event.call(this, type, options);
    this.detail = options && options.detail !== undefined ? options.detail : null;
  };

  globalThis.requestAnimationFrame = function(cb) { return setTimeout(cb, 16); };
  globalThis.cancelAnimationFrame = function(id) { clearTimeout(id); };

  globalThis.matchMedia = function() {
    return {
      matches: false, media: '',
      addEventListener: function() {}, removeEventListener: function() {},
      addListener: function() {}, removeListener: function() {}
    };
  };

  globalThis.getComputedStyle = function() { return new Proxy({}, { get: function() { return ''; } }); };

  globalThis.MutationObserver = globalThis.MutationObserver || function() {
    return { observe: function() {}, disconnect: function() {}, takeRecords: function() { return []; } };
  };

  globalThis.ResizeObserver = globalThis.ResizeObserver || function() {
    return { observe: function() {}, unobserve: function() {}, disconnect: function() {} };
  };

  globalThis.IntersectionObserver = globalThis.IntersectionObserver || function() {
    return { observe: function() {}, unobserve: function() {}, disconnect: function() {} };
  };
})();
