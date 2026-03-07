// TextEncoder/TextDecoder polyfill
// Uses native Rust functions __encodeUtf8 and __decodeUtf8 for the actual work
// __encodeUtf8(string) -> Array<number> (byte values)
// __decodeUtf8(Array<number>, fatal) -> string

(function() {
  globalThis.TextEncoder = class TextEncoder {
    get encoding() { return 'utf-8'; }

    encode(input) {
      if (input === undefined) return new Uint8Array(0);
      return new Uint8Array(__encodeUtf8(String(input)));
    }

    encodeInto(source, destination) {
      var encoded = this.encode(source);
      var written = Math.min(encoded.length, destination.length);
      var bytesWritten = written;
      if (bytesWritten < encoded.length && bytesWritten > 0) {
        while (bytesWritten > 0 && (encoded[bytesWritten] & 0xc0) === 0x80) {
          bytesWritten--;
        }
      }
      destination.set(encoded.subarray(0, bytesWritten));
      var read = 0;
      if (bytesWritten > 0) {
        var decoded = new TextDecoder().decode(encoded.subarray(0, bytesWritten));
        read = decoded.length;
      }
      return { read: read, written: bytesWritten };
    }
  };

  globalThis.TextDecoder = class TextDecoder {
    constructor(label, options) {
      if (label !== undefined) {
        var l = String(label).trim().toLowerCase();
        if (l !== 'utf-8' && l !== 'utf8' && l !== 'unicode-1-1-utf-8') {
          throw new RangeError('The encoding label provided must be utf-8');
        }
      }
      this._fatal = !!(options && options.fatal);
      this._ignoreBOM = !!(options && options.ignoreBOM);
    }

    get encoding() { return 'utf-8'; }
    get fatal() { return this._fatal; }
    get ignoreBOM() { return this._ignoreBOM; }

    decode(input, options) {
      if (input === undefined) return '';
      if (options && options.stream) {
        throw new Error('Streaming decode is not supported');
      }
      var bytes;
      if (input instanceof Uint8Array) {
        bytes = input;
      } else if (input instanceof ArrayBuffer) {
        bytes = new Uint8Array(input);
      } else if (ArrayBuffer.isView(input)) {
        bytes = new Uint8Array(input.buffer, input.byteOffset, input.byteLength);
      } else {
        throw new TypeError("The provided value is not of type '(ArrayBuffer or ArrayBufferView)'");
      }
      if (!this._ignoreBOM && bytes.length >= 3 &&
          bytes[0] === 0xEF && bytes[1] === 0xBB && bytes[2] === 0xBF) {
        bytes = bytes.subarray(3);
      }
      return __decodeUtf8(Array.from(bytes), this._fatal);
    }
  };
})();
