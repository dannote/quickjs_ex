// Buffer polyfill for QuickJS
// Based on extism/js-pdk (MIT), adapted for quickjs_ex

(function() {
  function normalizeEncoding(enc) {
    if (!enc) return 'utf8';
    switch (enc.toLowerCase()) {
      case 'utf8': case 'utf-8': return 'utf8';
      case 'ascii': return 'ascii';
      case 'latin1': case 'binary': return 'latin1';
      case 'base64': return 'base64';
      case 'base64url': return 'base64url';
      case 'hex': return 'hex';
      case 'ucs2': case 'ucs-2': case 'utf16le': case 'utf-16le': return 'utf16le';
      default: throw new TypeError('Unknown encoding: ' + enc);
    }
  }

  function encodingToBytes(str, encoding) {
    switch (encoding) {
      case 'utf8':
        return new TextEncoder().encode(str);
      case 'ascii':
      case 'latin1': {
        var bytes = new Uint8Array(str.length);
        for (var i = 0; i < str.length; i++) bytes[i] = str.charCodeAt(i) & 0xff;
        return bytes;
      }
      case 'base64': {
        var cleaned = str.replace(/[\s\r\n]+/g, '');
        while (cleaned.length % 4 !== 0) cleaned += '=';
        var binary = atob(cleaned);
        var bytes = new Uint8Array(binary.length);
        for (var i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
        return bytes;
      }
      case 'base64url': {
        var b64 = str.replace(/-/g, '+').replace(/_/g, '/');
        while (b64.length % 4 !== 0) b64 += '=';
        return encodingToBytes(b64, 'base64');
      }
      case 'hex': {
        if (str.length % 2 !== 0) throw new TypeError('Invalid hex string');
        var bytes = new Uint8Array(str.length / 2);
        for (var i = 0; i < str.length; i += 2)
          bytes[i / 2] = parseInt(str.substring(i, i + 2), 16);
        return bytes;
      }
      case 'utf16le': {
        var bytes = new Uint8Array(str.length * 2);
        for (var i = 0; i < str.length; i++) {
          var code = str.charCodeAt(i);
          bytes[i * 2] = code & 0xff;
          bytes[i * 2 + 1] = (code >> 8) & 0xff;
        }
        return bytes;
      }
      default: throw new TypeError('Unknown encoding: ' + encoding);
    }
  }

  function bytesToEncoding(bytes, encoding, start, end) {
    var slice = bytes.subarray(start, end);
    switch (encoding) {
      case 'utf8':
        return new TextDecoder().decode(slice);
      case 'ascii': {
        var s = '';
        for (var i = 0; i < slice.length; i++) s += String.fromCharCode(slice[i] & 0x7f);
        return s;
      }
      case 'latin1': {
        var s = '';
        for (var i = 0; i < slice.length; i++) s += String.fromCharCode(slice[i]);
        return s;
      }
      case 'base64': {
        var binary = '';
        for (var i = 0; i < slice.length; i++) binary += String.fromCharCode(slice[i]);
        return btoa(binary);
      }
      case 'base64url':
        return bytesToEncoding(bytes, 'base64', start, end)
          .replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
      case 'hex': {
        var hex = '';
        for (var i = 0; i < slice.length; i++)
          hex += (slice[i] < 16 ? '0' : '') + slice[i].toString(16);
        return hex;
      }
      case 'utf16le': {
        var s = '';
        for (var i = 0; i + 1 < slice.length; i += 2)
          s += String.fromCharCode(slice[i] | (slice[i + 1] << 8));
        return s;
      }
      default: throw new TypeError('Unknown encoding: ' + encoding);
    }
  }

  class _Buffer extends Uint8Array {
    static from(value, encodingOrOffset, length) {
      if (typeof value === 'string') {
        var encoding = normalizeEncoding(encodingOrOffset);
        var bytes = encodingToBytes(value, encoding);
        var buf = new _Buffer(bytes.length);
        buf.set(bytes);
        return buf;
      }
      if (value instanceof ArrayBuffer) {
        var offset = encodingOrOffset || 0;
        var len = length !== undefined ? length : value.byteLength - offset;
        return new _Buffer(value, offset, len);
      }
      if (_Buffer.isBuffer(value)) {
        var buf = new _Buffer(value.length);
        buf.set(value);
        return buf;
      }
      if (ArrayBuffer.isView(value)) {
        var src = new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
        var buf = new _Buffer(src.length);
        buf.set(src);
        return buf;
      }
      if (value && value.type === 'Buffer' && Array.isArray(value.data)) {
        return _Buffer.from(value.data);
      }
      if (Array.isArray(value) || (typeof value === 'object' && value !== null && typeof value.length === 'number')) {
        var buf = new _Buffer(value.length);
        for (var i = 0; i < value.length; i++) buf[i] = value[i] & 0xff;
        return buf;
      }
      throw new TypeError('The first argument must be a string, Buffer, ArrayBuffer, Array, or array-like object');
    }

    static alloc(size, fill, encoding) {
      var buf = new _Buffer(size);
      if (fill !== undefined) buf.fill(fill, 0, size, encoding);
      return buf;
    }

    static allocUnsafe(size) { return new _Buffer(size); }

    static isBuffer(obj) { return obj instanceof _Buffer; }

    static isEncoding(encoding) {
      try { normalizeEncoding(encoding); return true; } catch(e) { return false; }
    }

    static byteLength(string, encoding) {
      return encodingToBytes(string, normalizeEncoding(encoding)).length;
    }

    static concat(list, totalLength) {
      if (totalLength === undefined) {
        totalLength = 0;
        for (var i = 0; i < list.length; i++) totalLength += list[i].length;
      }
      var buf = _Buffer.alloc(totalLength);
      var offset = 0;
      for (var i = 0; i < list.length; i++) {
        if (offset + list[i].length > totalLength) {
          buf.set(list[i].subarray(0, totalLength - offset), offset);
          break;
        }
        buf.set(list[i], offset);
        offset += list[i].length;
      }
      return buf;
    }

    static compare(buf1, buf2) {
      var len = Math.min(buf1.length, buf2.length);
      for (var i = 0; i < len; i++) {
        if (buf1[i] < buf2[i]) return -1;
        if (buf1[i] > buf2[i]) return 1;
      }
      if (buf1.length < buf2.length) return -1;
      if (buf1.length > buf2.length) return 1;
      return 0;
    }

    toString(encoding, start, end) {
      var enc = normalizeEncoding(encoding);
      var s = start || 0;
      var e = end !== undefined ? end : this.length;
      return bytesToEncoding(this, enc, s, e);
    }

    toJSON() { return { type: 'Buffer', data: Array.from(this) }; }

    equals(other) {
      if (this.length !== other.length) return false;
      for (var i = 0; i < this.length; i++) if (this[i] !== other[i]) return false;
      return true;
    }

    compare(target, targetStart, targetEnd, sourceStart, sourceEnd) {
      var tStart = targetStart || 0;
      var tEnd = targetEnd !== undefined ? targetEnd : target.length;
      var sStart = sourceStart || 0;
      var sEnd = sourceEnd !== undefined ? sourceEnd : this.length;
      var src = this.subarray(sStart, sEnd);
      var tgt = target.subarray(tStart, tEnd);
      var len = Math.min(src.length, tgt.length);
      for (var i = 0; i < len; i++) {
        if (src[i] < tgt[i]) return -1;
        if (src[i] > tgt[i]) return 1;
      }
      if (src.length < tgt.length) return -1;
      if (src.length > tgt.length) return 1;
      return 0;
    }

    copy(target, targetStart, sourceStart, sourceEnd) {
      var tStart = targetStart || 0;
      var sStart = sourceStart || 0;
      var sEnd = sourceEnd !== undefined ? sourceEnd : this.length;
      var src = this.subarray(sStart, sEnd);
      var toCopy = Math.min(src.length, target.length - tStart);
      target.set(src.subarray(0, toCopy), tStart);
      return toCopy;
    }

    write(string, offsetOrEncoding, lengthOrEncoding, encoding) {
      var off = 0, enc = 'utf8', len;
      if (typeof offsetOrEncoding === 'string') {
        enc = normalizeEncoding(offsetOrEncoding);
      } else if (typeof offsetOrEncoding === 'number') {
        off = offsetOrEncoding;
        if (typeof lengthOrEncoding === 'string') {
          enc = normalizeEncoding(lengthOrEncoding);
        } else if (typeof lengthOrEncoding === 'number') {
          len = lengthOrEncoding;
          if (encoding) enc = normalizeEncoding(encoding);
        }
      }
      var bytes = encodingToBytes(string, enc);
      var maxLen = this.length - off;
      var toCopy = Math.min(len !== undefined ? Math.min(len, bytes.length) : bytes.length, maxLen);
      this.set(bytes.subarray(0, toCopy), off);
      return toCopy;
    }

    slice(start, end) {
      var sub = super.subarray(start, end);
      Object.setPrototypeOf(sub, _Buffer.prototype);
      return sub;
    }

    indexOf(value, byteOffset, encoding) {
      var offset = byteOffset || 0;
      if (typeof value === 'number') {
        for (var i = offset; i < this.length; i++) if (this[i] === (value & 0xff)) return i;
        return -1;
      }
      var needle;
      if (typeof value === 'string') needle = encodingToBytes(value, normalizeEncoding(encoding));
      else needle = value;
      if (needle.length === 0) return offset;
      for (var i = offset; i <= this.length - needle.length; i++) {
        var found = true;
        for (var j = 0; j < needle.length; j++) {
          if (this[i + j] !== needle[j]) { found = false; break; }
        }
        if (found) return i;
      }
      return -1;
    }

    includes(value, byteOffset, encoding) {
      return this.indexOf(value, byteOffset, encoding) !== -1;
    }

    fill(value, offset, end, encoding) {
      var off = offset || 0;
      var e = end !== undefined ? end : this.length;
      if (typeof value === 'number') {
        super.fill(value & 0xff, off, e);
        return this;
      }
      if (typeof value === 'string') {
        if (value.length === 0) { super.fill(0, off, e); return this; }
        var enc = normalizeEncoding(encoding);
        var bytes = encodingToBytes(value, enc);
        if (bytes.length === 1) { super.fill(bytes[0], off, e); return this; }
        for (var i = off; i < e; i++) this[i] = bytes[(i - off) % bytes.length];
        return this;
      }
      if (value instanceof Uint8Array) {
        for (var i = off; i < e; i++) this[i] = value[(i - off) % value.length];
        return this;
      }
      throw new TypeError('value must be a number, string, Buffer, or Uint8Array');
    }

    // Read methods
    readUInt8(offset) { return this[offset || 0]; }
    readUInt16BE(offset) { offset = offset || 0; return (this[offset] << 8) | this[offset + 1]; }
    readUInt16LE(offset) { offset = offset || 0; return this[offset] | (this[offset + 1] << 8); }
    readUInt32BE(offset) { offset = offset || 0; return ((this[offset] * 0x1000000 + ((this[offset+1] << 16) | (this[offset+2] << 8) | this[offset+3])) >>> 0); }
    readUInt32LE(offset) { offset = offset || 0; return ((this[offset] | (this[offset+1] << 8) | (this[offset+2] << 16) | (this[offset+3] * 0x1000000)) >>> 0); }
    readInt8(offset) { var v = this[offset || 0]; return v & 0x80 ? v - 0x100 : v; }
    readInt16BE(offset) { offset = offset || 0; var v = (this[offset] << 8) | this[offset+1]; return v & 0x8000 ? v - 0x10000 : v; }
    readInt16LE(offset) { offset = offset || 0; var v = this[offset] | (this[offset+1] << 8); return v & 0x8000 ? v - 0x10000 : v; }
    readInt32BE(offset) { offset = offset || 0; return (this[offset] << 24) | (this[offset+1] << 16) | (this[offset+2] << 8) | this[offset+3]; }
    readInt32LE(offset) { offset = offset || 0; return this[offset] | (this[offset+1] << 8) | (this[offset+2] << 16) | (this[offset+3] << 24); }
    readFloatBE(offset) { var dv = new DataView(this.buffer, this.byteOffset, this.byteLength); return dv.getFloat32(offset || 0, false); }
    readFloatLE(offset) { var dv = new DataView(this.buffer, this.byteOffset, this.byteLength); return dv.getFloat32(offset || 0, true); }
    readDoubleBE(offset) { var dv = new DataView(this.buffer, this.byteOffset, this.byteLength); return dv.getFloat64(offset || 0, false); }
    readDoubleLE(offset) { var dv = new DataView(this.buffer, this.byteOffset, this.byteLength); return dv.getFloat64(offset || 0, true); }

    // Write methods
    writeUInt8(value, offset) { offset = offset || 0; this[offset] = value & 0xff; return offset + 1; }
    writeUInt16BE(value, offset) { offset = offset || 0; this[offset] = (value >>> 8) & 0xff; this[offset+1] = value & 0xff; return offset + 2; }
    writeUInt16LE(value, offset) { offset = offset || 0; this[offset] = value & 0xff; this[offset+1] = (value >>> 8) & 0xff; return offset + 2; }
    writeUInt32BE(value, offset) { offset = offset || 0; this[offset] = (value >>> 24) & 0xff; this[offset+1] = (value >>> 16) & 0xff; this[offset+2] = (value >>> 8) & 0xff; this[offset+3] = value & 0xff; return offset + 4; }
    writeUInt32LE(value, offset) { offset = offset || 0; this[offset] = value & 0xff; this[offset+1] = (value >>> 8) & 0xff; this[offset+2] = (value >>> 16) & 0xff; this[offset+3] = (value >>> 24) & 0xff; return offset + 4; }
    writeInt8(value, offset) { offset = offset || 0; this[offset] = value & 0xff; return offset + 1; }
    writeInt16BE(value, offset) { offset = offset || 0; this[offset] = (value >>> 8) & 0xff; this[offset+1] = value & 0xff; return offset + 2; }
    writeInt16LE(value, offset) { offset = offset || 0; this[offset] = value & 0xff; this[offset+1] = (value >>> 8) & 0xff; return offset + 2; }
    writeInt32BE(value, offset) { offset = offset || 0; this[offset] = (value >>> 24) & 0xff; this[offset+1] = (value >>> 16) & 0xff; this[offset+2] = (value >>> 8) & 0xff; this[offset+3] = value & 0xff; return offset + 4; }
    writeInt32LE(value, offset) { offset = offset || 0; this[offset] = value & 0xff; this[offset+1] = (value >>> 8) & 0xff; this[offset+2] = (value >>> 16) & 0xff; this[offset+3] = (value >>> 24) & 0xff; return offset + 4; }
    writeFloatBE(value, offset) { offset = offset || 0; var dv = new DataView(this.buffer, this.byteOffset, this.byteLength); dv.setFloat32(offset, value, false); return offset + 4; }
    writeFloatLE(value, offset) { offset = offset || 0; var dv = new DataView(this.buffer, this.byteOffset, this.byteLength); dv.setFloat32(offset, value, true); return offset + 4; }
    writeDoubleBE(value, offset) { offset = offset || 0; var dv = new DataView(this.buffer, this.byteOffset, this.byteLength); dv.setFloat64(offset, value, false); return offset + 8; }
    writeDoubleLE(value, offset) { offset = offset || 0; var dv = new DataView(this.buffer, this.byteOffset, this.byteLength); dv.setFloat64(offset, value, true); return offset + 8; }
  }

  globalThis.Buffer = _Buffer;
})();
