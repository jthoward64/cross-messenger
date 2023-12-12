// @ts-nocheck
export type Result<T, E> = { tag: 'ok', val: T } | { tag: 'err', val: E };
class Deserializer {
    source
    offset
    
    constructor(bytes) {
        this.source = bytes
        this.offset = 0
    }

    pop() {
        return this.source[this.offset++]
    }

    try_take_n(len) {
        const out = this.source.slice(this.offset, this.offset + len)
        this.offset += len
        return out
    }
}
// function varint_max(bits) {
//   const BITS_PER_BYTE = 8;
//   const BITS_PER_VARINT_BYTE = 7;

//   const roundup_bits = bits + (BITS_PER_BYTE - 1);

//   return Math.floor(roundup_bits / BITS_PER_VARINT_BYTE);
// }

const varint_max = {
  16: 3,
  32: 5,
  64: 10,
  128: 19
}
function max_of_last_byte(type) {
  let extra_bits = type % 7;
  return (1 << extra_bits) - 1;
}

function de_varint(de, bits) {
  let out = 0;

  for (let i = 0; i < varint_max[bits]; i++) {
    const val = de.pop();
    const carry = val & 0x7F;
    out |= carry << (7 * i);

    if ((val & 0x80) === 0) {
      if (i === varint_max[bits] - 1 && val > max_of_last_byte(bits)) {
        throw new Error('deserialize bad variant')
      } else {
        return out
      }
    }
  }

  throw new Error('deserialize bad variant')
}

function de_varint_big(de, bits) {
  let out = 0n;

  for (let i = 0; i < varint_max[bits]; i++) {
    const val = de.pop();
    const carry = BigInt(val) & 0x7Fn;
    out |= carry << (7n * BigInt(i));

    if ((val & 0x80) === 0) {
      if (i === varint_max[bits] - 1 && val > max_of_last_byte(bits)) {
        throw new Error('deserialize bad variant')
      } else {
        return out
      }
    }
  }

  throw new Error('deserialize bad variant')
}
function deserializeU32(de) {
    return de_varint(de, 32)
}
function deserializeU64(de) {
  return de_varint_big(de, 64)
}
function deserializeString(de) {
    const sz = deserializeU64(de);

    let bytes = de.try_take_n(Number(sz));

    return __text_decoder.decode(bytes);
}
function deserializeOption(de, inner) {
    const tag = de.pop()

    switch (tag) {
        case 0:
            return null
        case 1: 
            return inner(de)
        default:
            throw new Error(`Deserialize bad option ${tag}`)
    }
}
function deserializeResult(de, ok, err) {
    const tag = de.pop()

    switch (tag) {
        case 0:
            return { tag: 'ok', val: ok(de) }
        case 1: 
            return { tag: 'err', val: err(de) }
        default:
            throw new Error(`Deserialize bad result ${tag}`)
    }
}
function deserializeList(de, inner) {
    const len = deserializeU64(de);

    let out = [];

    for (let i = 0; i < len; i++) {
        out.push(inner(de));   
    }

    return out;
}
function ser_varint(out, bits, val) {
  let buf = []
  for (let i = 0; i < varint_max[bits]; i++) {
    const buffer = new ArrayBuffer(bits / 8);
    const view = new DataView(buffer);
    view.setInt16(0, val, true);
    buf[i] = view.getUint8(0);
    if (val < 128) {
      out.push(...buf)
      return;
    }

    buf[i] |= 0x80;
    val >>= 7;
  }
  out.push(...buf)
}

function ser_varint_big(out, bits, val) {
  let buf = []
  for (let i = 0; i < varint_max[bits]; i++) {
    const buffer = new ArrayBuffer(bits / 8);
    const view = new DataView(buffer);
    view.setInt16(0, Number(val), true);
    buf[i] = view.getUint8(0);
    if (val < 128) {
      out.push(...buf)
      return;
    }

    buf[i] |= 0x80;
    val >>= 7n;
  }
  out.push(...buf)
}
function serializeU32(out, val) {
    return ser_varint(out, 32, val)
}
function serializeU64(out, val) {
  return ser_varint_big(out, 64, BigInt(val))
}
function serializeString(out, val) {
    serializeU64(out, val.length);

    out.push(...__text_encoder.encode(val))
}
function serializeOption(out, inner, val) {
    serializeU8(out, !!val ? 1 : 0)
    if (val) {
        inner(out, val)
    }
}
function serializeResult(out, ok, err, val) {
    if (val.Ok) {
        serializeU8(out, 0);
        return ok(out, val.Ok);
    }

    if (val.Err) {
        serializeU8(out, 1);
        return err(out, val.Err);
    }

    throw new Error(`Serialize bad result ${val}`);
}
function serializeList(out, inner, val) {
    serializeU64(out, val.length)
    for (const el of val) {
        inner(out, el)
    }
}
const __text_decoder = new TextDecoder('utf-8');
const __text_encoder = new TextEncoder();
function deserializeLoginErrorCode(de) {
    const tag = deserializeU32(de)

    switch (tag) {
        case 0:
    return "TwoFactorRequired"
case 1:
    return "LoginFailed"
case 2:
    return "Unknown"

        default:
            throw new Error(`unknown enum case ${tag}`)
    }
}function deserializeLogoutErrorCode(de) {
    const tag = deserializeU32(de)

    switch (tag) {
        case 0:
    return "NotLoggedIn"
case 1:
    return "Unknown"

        default:
            throw new Error(`unknown enum case ${tag}`)
    }
}function deserializeUser(de) {
    return {
        userId: deserializeString(de),
handles: deserializeList(de, (de) => deserializeString(de)),
selectedHandle: deserializeString(de)
    }
}function deserializeGetUserErrorCode(de) {
    const tag = deserializeU32(de)

    switch (tag) {
        case 0:
    return "NotLoggedIn"
case 1:
    return "Unknown"

        default:
            throw new Error(`unknown enum case ${tag}`)
    }
}function deserializeSelectHandleErrorCode(de) {
    const tag = deserializeU32(de)

    switch (tag) {
        case 0:
    return "NotLoggedIn"
case 1:
    return "HandleNotFound"
case 2:
    return "Unknown"

        default:
            throw new Error(`unknown enum case ${tag}`)
    }
}

export enum LoginErrorCode { 
TwoFactorRequired,

LoginFailed,

Unknown,
 }

export enum LogoutErrorCode { 
NotLoggedIn,

Unknown,
 }

export interface User { 
userId: string,

handles: string[],

selectedHandle: string,
 }

export enum GetUserErrorCode { 
NotLoggedIn,

Unknown,
 }

export enum SelectHandleErrorCode { 
NotLoggedIn,

HandleNotFound,

Unknown,
 }



export async function login (username: string, password: string, code: string | null) : Promise<LoginErrorCode | null> {
    const out = []
    serializeString(out, username);
serializeString(out, password);
serializeOption(out, (out, v) => serializeString(out, v), code)

    return fetch('ipc://localhost/ipc/login', { method: "POST", body: Uint8Array.from(out) })
        .then(r => r.arrayBuffer())
        .then(bytes => {
            const de = new Deserializer(new Uint8Array(bytes))

            return deserializeOption(de, (de) => deserializeLoginErrorCode(de))
        }) as Promise<LoginErrorCode | null>
}
        

export async function logout () : Promise<LogoutErrorCode | null> {
    const out = []
    

    return fetch('ipc://localhost/ipc/logout', { method: "POST", body: Uint8Array.from(out) })
        .then(r => r.arrayBuffer())
        .then(bytes => {
            const de = new Deserializer(new Uint8Array(bytes))

            return deserializeOption(de, (de) => deserializeLogoutErrorCode(de))
        }) as Promise<LogoutErrorCode | null>
}
        

export async function getUser () : Promise<Result<User, GetUserErrorCode>> {
    const out = []
    

    return fetch('ipc://localhost/ipc/get_user', { method: "POST", body: Uint8Array.from(out) })
        .then(r => r.arrayBuffer())
        .then(bytes => {
            const de = new Deserializer(new Uint8Array(bytes))

            return deserializeResult(de, (de) => deserializeUser(de), (de) => deserializeGetUserErrorCode(de))
        }) as Promise<Result<User, GetUserErrorCode>>
}
        

export async function selectHandle (handle: string) : Promise<SelectHandleErrorCode | null> {
    const out = []
    serializeString(out, handle)

    return fetch('ipc://localhost/ipc/select_handle', { method: "POST", body: Uint8Array.from(out) })
        .then(r => r.arrayBuffer())
        .then(bytes => {
            const de = new Deserializer(new Uint8Array(bytes))

            return deserializeOption(de, (de) => deserializeSelectHandleErrorCode(de))
        }) as Promise<SelectHandleErrorCode | null>
}
        