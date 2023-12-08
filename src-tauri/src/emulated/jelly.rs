use std::*;
use std::collections::HashMap;

use io::BytesIO;
const STOP_ADDRESS: _ = 9437184;
struct unicorn {

}

impl unicorn {
struct x86_const {

}

impl x86_const {
const UC_X86_REG_RAX: _ = 0;
const UC_X86_REG_RBX: _ = 1;
const UC_X86_REG_RCX: _ = 2;
const UC_X86_REG_RDX: _ = 3;
const UC_X86_REG_RSI: _ = 4;
const UC_X86_REG_RDI: _ = 5;
const UC_X86_REG_RBP: _ = 6;
const UC_X86_REG_RSP: _ = 7;
const UC_X86_REG_R8: _ = 8;
const UC_X86_REG_R9: _ = 9;
const UC_X86_REG_R10: _ = 10;
const UC_X86_REG_R11: _ = 11;
const UC_X86_REG_R12: _ = 12;
const UC_X86_REG_R13: _ = 13;
const UC_X86_REG_R14: _ = 14;
const UC_X86_REG_R15: _ = 15;
const UC_X86_REG_RIP: _ = 16;
const UC_X86_REG_EFLAGS: _ = 17;
const UC_X86_REG_CS: _ = 18;
const UC_X86_REG_SS: _ = 19;
const UC_X86_REG_DS: _ = 20;
const UC_X86_REG_ES: _ = 21;
const UC_X86_REG_FS: _ = 22;
const UC_X86_REG_GS: _ = 23; 
} 
}
// let ARG_REGISTERS = vec![unicorn::x86_const.UC_X86_REG_RDI, unicorn::x86_const.UC_X86_REG_RSI, unicorn::x86_const.UC_X86_REG_RDX, unicorn::x86_const.UC_X86_REG_RCX, unicorn::x86_const.UC_X86_REG_R8, unicorn::x86_const.UC_X86_REG_R9];
struct VirtualInstructions {
uc: unicorn::Uc,
}

impl VirtualInstructions {
fn __init__(&self, uc: unicorn::Uc)  {
self.uc = uc;
}
fn push(&self, value: i32)  {
self.uc.reg_write(unicorn::x86_const.UC_X86_REG_ESP, (self.uc.reg_read(unicorn::x86_const.UC_X86_REG_ESP) - 8));
self.uc.mem_write(self.uc.reg_read(unicorn::x86_const.UC_X86_REG_ESP), value.to_bytes(8, "little"));
}
fn pop(&self) -> i32 {
let value = int.from_bytes(self.uc.mem_read(self.uc.reg_read(unicorn::x86_const.UC_X86_REG_ESP), 8), "little");
self.uc.reg_write(unicorn::x86_const.UC_X86_REG_ESP, (self.uc.reg_read(unicorn::x86_const.UC_X86_REG_ESP) + 8));
return value;
}
fn _set_args(&self, args: list<i32>)  {
for i in (0..args.len()) {
if i < 6 {
self.uc.reg_write(ARG_REGISTERS[i], args[i]);
} else {
self.push(args[i]);
}
}
}
fn call<RT>(&self, address: i32, args: list<i32>) -> RT {
self.push(STOP_ADDRESS);
self._set_args(args);
self.uc.emu_start(address, STOP_ADDRESS);
return self.uc.reg_read(unicorn::x86_const.UC_X86_REG_RAX);
} 
}
struct Jelly {
_binary: &[u8],
_hooks: dict<&str, callable>,
instr: ST0,
uc: ST1,
_resolved_hooks: HashMap<_,_>,
}

impl Jelly {
const UC_ARCH: _ = unicorn::UC_ARCH_X86;
const UC_MODE: _ = unicorn::UC_MODE_64;
const BINARY_BASE: _ = 0;
const HOOK_BASE: _ = 13631488;
const HOOK_SIZE: _ = 4096;
const STACK_BASE: _ = 3145728;
const STACK_SIZE: _ = 1048576;
const HEAP_BASE: _ = 4194304;
const HEAP_SIZE: _ = 1048576;
const STOP_ADDRESS: _ = 9437184;
// let _hooks: dict[(str, callable)] = HashMap::new();
// "Symbol name to hook function mapping";
// let instr: VirtualInstructions = None;
// let uc: unicorn::Uc = None;
// let _binary: bytes = b"";
// let _heap_use: int = 0;
fn __init__(&self, binary: &[u8])  {
self._binary = binary;
}
fn setup(&self, hooks: dict<&str, callable>)  {
self._hooks = hooks;
self._setup_unicorn();
self.instr = VirtualInstructions(self.uc);
self._setup_hooks();
self._map_binary();
self._setup_stack();
self._setup_heap();
self._setup_stop();
}
fn _setup_unicorn(&self)  {
self.uc = unicorn::Uc(self.UC_ARCH, self.UC_MODE);
}
fn _setup_stack(&self)  {
self.uc.mem_map(self.STACK_BASE, self.STACK_SIZE);
self.uc.mem_write(self.STACK_BASE, (b"\x00"*self.STACK_SIZE));
self.uc.reg_write(unicorn::x86_const.UC_X86_REG_ESP, (self.STACK_BASE + self.STACK_SIZE));
self.uc.reg_write(unicorn::x86_const.UC_X86_REG_EBP, (self.STACK_BASE + self.STACK_SIZE));
}
fn _setup_heap(&self)  {
self.uc.mem_map(self.HEAP_BASE, self.HEAP_SIZE);
self.uc.mem_write(self.HEAP_BASE, (b"\x00"*self.HEAP_SIZE));
}
fn wrap_hook(&self, func: callable) -> callable {
let arg_count = func.__code__.co_argcount;
fn wrapper(&self)  {
let mut args = vec![];
for i in (1..arg_count) {
if i < 6 {
args.push(self.uc.reg_read(ARG_REGISTERS[(i - 1)]));
} else {
args.push(self.instr.pop());
}
}
if args != vec![] {
1;
}
let ret = func(self, starred!(args)/*unsupported*/);
if ret != None {
self.uc.reg_write(unicorn::x86_const.UC_X86_REG_RAX, ret);
}
return;
}
return wrapper;
}
fn malloc(&self, size: i32) -> i32 {
let addr = (self.HEAP_BASE + self._heap_use);
self._heap_use += size;
return addr;
}
fn _setup_stop(&self)  {
self.uc.mem_map(self.STOP_ADDRESS, 4096);
self.uc.mem_write(self.STOP_ADDRESS, (b"\xc3"*4096));
}
fn _resolve_hook(uc: unicorn::Uc, address: i32, size: i32, self: None)  {
for (name, addr) in self._resolved_hooks.items() {
if addr == address {
self._hooks[name](self);
}
}
}
fn _setup_hooks(&self)  {
for (name, func) in self._hooks.items() {
self._hooks[name] = self.wrap_hook(func);
}
self.uc.mem_map(self.HOOK_BASE, self.HOOK_SIZE);
self.uc.mem_write(self.HOOK_BASE, (b"\xc3"*self.HOOK_SIZE));
let mut current_address = self.HOOK_BASE;
self._resolved_hooks = HashMap::new();
for hook in self._hooks {
self._resolved_hooks[hook] = current_address;
current_address += 1;
}
self.uc.hook_add(unicorn::UC_HOOK_CODE, Jelly::_resolve_hook, self.HOOK_BASE, (self.HOOK_BASE + self.HOOK_SIZE), self);
}
fn _map_binary(&self)  {
self.uc.mem_map(self.BINARY_BASE, round_to_page_size(self._binary.len(), self.uc.ctl_get_page_size()));
self.uc.mem_write(self.BINARY_BASE, self._binary);
self.uc.mem_unmap(0, self.uc.ctl_get_page_size());
}
// fn _do_bind<T0, T1, T2>(&self, mu: unicorn::Uc, type: T0, location: T1, name: T2)  {
// if type_ == 1 {
// if self._hooks.iter().any(|&x| x == name) {
// mu.mem_write(location, self._resolved_hooks[name].to_bytes(8, "little"));
// } else {
// /*pass*/
// }
// } else {
// raise!(NotImplementedError()); //unsupported
// }
// }
fn _parse_lazy_binds<T0, T1, T2, T3, T4>(&self, mu: unicorn::Uc, indirect_offset: T0, section: T1, dysimtab: T2, strtab: T3, symtab: T4)  {
for i in (0..i32::from((section["size"]/8))) {
let mut dysym = dysimtab[((indirect_offset + i)*4)..(((indirect_offset + i)*4) + 4)];
dysym = int.from_bytes(dysym, "little");
let index = (dysym & 1073741823);
let symbol = symtab[(index*16)..((index*16) + 4)];
let strx = int.from_bytes(symbol, "little");
let name = c_string(strtab, strx);
self._do_bind(mu, 1, (section["offset"] + (i*8)), name);
}
}
fn _parse_binds<T0>(&self, mu: unicorn::Uc, binds: &[u8], segments: T0)  {
let blen = binds.len();
let binds: BytesIO = BytesIO(binds);
let mut ordinal = 0;
let mut symbolName = "";
let mut type_ = BIND_TYPE_POINTER;
let addend = 0;
let mut segIndex = 0;
let mut segOffset = 0;
while binds.tell() < blen {
let current = binds.read(1)[0];
let opcode = (current & BIND_OPCODE_MASK);
let immediate = (current & BIND_IMMEDIATE_MASK);
if opcode == BIND_OPCODE_DONE {
break;
} else {
if opcode == BIND_OPCODE_SET_DYLIB_ORDINAL_IMM {
ordinal = immediate;
} else {
if opcode == BIND_OPCODE_SET_DYLIB_ORDINAL_ULEB {
ordinal = decodeULEB128(binds);
} else {
if opcode == BIND_OPCODE_SET_DYLIB_SPECIAL_IMM {
if immediate == 0 {
ordinal = 0;
} else {
ordinal = (BIND_OPCODE_MASK | immediate);
}
} else {
if opcode == BIND_OPCODE_SET_SYMBOL_TRAILING_FLAGS_IMM {
symbolName = "";
while true {
let b = binds.read(1)[0];
if b == 0 {
break;
}
symbolName += chr(b);
}
} else {
if opcode == BIND_OPCODE_SET_TYPE_IMM {
type_ = immediate;
} else {
if opcode == BIND_OPCODE_SET_ADDEND_SLEB {
raise!(NotImplementedError("BIND_OPCODE_SET_ADDEND_SLEB")); //unsupported
} else {
if opcode == BIND_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB {
segIndex = immediate;
segOffset = decodeULEB128(binds);
} else {
if opcode == BIND_OPCODE_ADD_ADDR_ULEB {
segOffset += decodeULEB128(binds);
} else {
if opcode == BIND_OPCODE_DO_BIND {
self._do_bind(mu, type_, (segments[segIndex]["offset"] + segOffset), symbolName);
segOffset += 8;
} else {
if opcode == BIND_OPCODE_DO_BIND_ADD_ADDR_ULEB {
self._do_bind(mu, type_, (segments[segIndex]["offset"] + segOffset), symbolName);
segOffset += (decodeULEB128(binds) + 8);
} else {
if opcode == BIND_OPCODE_DO_BIND_ADD_ADDR_IMM_SCALED {
self._do_bind(mu, type_, (segments[segIndex]["offset"] + segOffset), symbolName);
segOffset += ((immediate*8) + 8);
} else {
if opcode == BIND_OPCODE_DO_BIND_ULEB_TIMES_SKIPPING_ULEB {
let count = decodeULEB128(binds);
let skip = decodeULEB128(binds);
for i in (0..count) {
self._do_bind(mu, type_, (segments[segIndex]["offset"] + segOffset), symbolName);
segOffset += (skip + 8);
}
} else {
raise!(NotImplementedError()); //unsupported
}
}
}
}
}
}
}
}
}
}
}
}
}
}
} 
}
const BIND_OPCODE_DONE: _ = 0;
const BIND_OPCODE_SET_DYLIB_ORDINAL_IMM: _ = 16;
const BIND_OPCODE_SET_DYLIB_ORDINAL_ULEB: _ = 32;
const BIND_OPCODE_SET_DYLIB_SPECIAL_IMM: _ = 48;
const BIND_OPCODE_SET_SYMBOL_TRAILING_FLAGS_IMM: _ = 64;
const BIND_OPCODE_SET_TYPE_IMM: _ = 80;
const BIND_OPCODE_SET_ADDEND_SLEB: _ = 96;
const BIND_OPCODE_SET_SEGMENT_AND_OFFSET_ULEB: _ = 112;
const BIND_OPCODE_ADD_ADDR_ULEB: _ = 128;
const BIND_OPCODE_DO_BIND: _ = 144;
const BIND_OPCODE_DO_BIND_ADD_ADDR_ULEB: _ = 160;
const BIND_OPCODE_DO_BIND_ADD_ADDR_IMM_SCALED: _ = 176;
const BIND_OPCODE_DO_BIND_ULEB_TIMES_SKIPPING_ULEB: _ = 192;
const BIND_OPCODE_THREADED: _ = 208;
const BIND_TYPE_POINTER: _ = 1;
const BIND_OPCODE_MASK: _ = 240;
const BIND_IMMEDIATE_MASK: _ = 15;
fn round_to_page_size(size: i32, page_size: i32) -> i32 {
return (((size + page_size) - 1) & None(page_size - 1));
}
fn decodeULEB128(bytes: BytesIO) -> i32 {
let mut result = 0;
let mut shift = 0;
while true {
let b = bytes.read(1)[0];
result |= ((b & 127) << shift);
if (b & 128) == 0 {
break;
}
shift += 7;
}
return result;
}
fn c_string<T0>(bytes: T0, start: i32) -> &str {
let mut out = "";
let mut i = start;
while true {
if i > bytes.len()||bytes[i] == 0 {
break;
}
out += chr(bytes[i]);
i += 1;
}
return out;
}