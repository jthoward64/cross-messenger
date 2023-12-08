use std::collections::HashMap;
use std::*;

/*
Modified from https://github.com/aaronst/macholibre/blob/master/macholibre/parser.py

This file is Copyright 2016 Aaron Stephens <aaronjst93@gmail.com>

Licensed under the Apache License, Version 2.0 (the \"License\");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an \"AS IS\" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/
use collections::Counter;
use datetime::datetime;
use io::BytesIO;
use json::dump;
use math::{exp, log};
use os::SEEK_END;
use plistlib::loads;
use re::split;
use uuid::UUID;
const logger: _ = logging.getLogger("jelly");
struct Parser {
    __extract_certs: bool,
    __file: ST0,
    __is_64_bit: bool,
    __is_little_endian: bool,
    __macho: HashMap<_, _>,
    __output: ST1,
    file: ST2,
    segments: Vec<_>,
    symtab: ST3,
    dyld_info: ST4,
    dysymtab: ST5,
}

impl Parser {
    /*
    Main object containing all the necessary functions to parse a mach-o binary.
    */
    fn __init__<T0>(&self, file: T0) {
        "Initialize instance variables and flags.";
        self.__extract_certs = false;
        self.__file = BytesIO(file);
        self.__is_64_bit = true;
        self.__is_little_endian = true;
        self.__macho = HashMap::new();
        self.__output = [("name", "IMDAppleServices")]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        self.file = self.__file;
    }
    fn add_abnormality<T0>(&self, abnormality: T0) {
        "Add abnormality to output.";
        if self.__output.iter().all(|&x| x != "abnormalities") {
            self.__output["abnormalities"] = vec![];
        }
        self.__output["abnormalities"].append(abnormality);
    }
    fn calc_entropy<T0, RT>(&self, b: T0) -> RT {
        "Calculate byte entropy for given bytes.";
        let byte_counts = Counter();
        let mut entropy = 0;
        for i in b {
            byte_counts[i] += 1;
        }
        let total = float(byte_counts.values().iter().sum());
        for count in byte_counts.values() {
            let p = (float(count) / total);
            entropy -= (p * log(p, 256));
        }
        return entropy;
    }
    fn get_string<RT>(&self) -> RT {
        "Read a null-terminated string from macho.";
        let mut string = bytearray();
        let mut c = self.__file.read(1);
        while (b"\x00", "").iter().all(|&x| x != c) {
            string += c;
            c = self.__file.read(1);
        }
        return string.decode("utf-8", "replace");
    }
    fn get_int<T0, RT>(&self, ignore_endian: T0) -> RT {
        "Read a 4-byte integer from macho, account for endian-ness.";
        let integer = self.__file.read(4);
        if self.__is_little_endian && !ignore_endian {
            return int.from_bytes(integer, "little");
        }
        return int.from_bytes(integer, "big");
    }
    fn get_ll<RT>(&self) -> RT {
        "Read an 8-byte long long from macho, account for endian-ness.";
        let longlong = self.__file.read(8);
        if self.__is_little_endian {
            return int.from_bytes(longlong, "little");
        }
        return int.from_bytes(longlong, "big");
    }
    fn make_version<T0, RT>(&self, version: T0) -> RT {
        "Construct a version number from given bytes.";
        let vx = (version >> 16);
        let vy = ((version >> 8) & 255);
        let vz = (version & 255);
        return "{}.{}.{}".format(vx, vy, vz);
    }
    fn identify_file<RT>(&self) -> RT {
        "Identify if the given file is a single Mach-O or a
        Universal binary.";
        let magic = self.get_int(true);
        if mdictionary::machos.iter().any(|&x| x == magic) {
            return mdictionary::machos[magic];
        } else {
            raise!(ValueError(
                "Provided file has unrecognized magic: {}".format(magic)
            )); //unsupported
        }
    }
    fn parse_macho_flags<T0, RT>(&self, flags: T0) -> RT {
        "Parse ``flags`` into list of readable flags.";
        let mut output = vec![];
        let mut i = 0;
        while i < 28 {
            if (1 & (flags >> i)) == 1 {
                if mdictionary::flags.iter().any(|&x| x == 2.pow(i)) {
                    output.push(mdictionary::flags[2.pow(i)]);
                } else {
                    self.add_abnormality("Unknown mach-o flag \"{}\".".format(2.pow(i)));
                }
            }
            i += 1;
        }
        return output;
    }
    fn get_segment_entropy<T0, T1, T2, RT>(&self, m_offset: T0, offset: T1, size: T2) -> RT {
        "Determine byte-entropy for this segment.";
        let old = self.__file.tell();
        self.__file.seek((m_offset + offset));
        let entropy = self.calc_entropy(self.__file.read(size));
        self.__file.seek(old);
        return entropy;
    }
    fn parse_section_attrs<T0, RT>(&self, attrs: T0) -> RT {
        "Parse section attributes.";
        let mut output = vec![];
        for a in mdictionary::section_attrs {
            if (attrs & a) == a {
                output.push(mdictionary::section_attrs[a]);
            }
        }
        return output;
    }
    fn parse_section_flags<T0, T1>(&self, output: T0, flags: T1) {
        "Parse section flags into section type and attributes.";
        output["type"] = mdictionary::section_types[(flags & 255)];
        let attrs = (flags & 4294967040);
        output["attrs"] = self.parse_section_attrs(attrs);
    }
    fn parse_section<RT>(&self) -> RT {
        "Parse section.";
        let name = self.__file.read(16).decode().rstrip(" ");
        let segname = self.__file.read(16).decode().rstrip(" ");
        let addr = if self.__is_64_bit {
            self.get_ll()
        } else {
            self.get_int()
        };
        let size = if self.__is_64_bit {
            self.get_ll()
        } else {
            self.get_int()
        };
        let offset = self.get_int();
        let align = self.get_int();
        let reloff = self.get_int();
        let nreloc = self.get_int();
        let flags = self.get_int();
        let r1 = self.get_int();
        let r2 = self.get_int();
        let r3 = self.get_int();
        let output = [
            ("name", name),
            ("segname", segname),
            ("addr", addr),
            ("offset", offset),
            ("align", align),
            ("reloff", reloff),
            ("nreloc", nreloc),
            ("size", size),
            ("r1", r1),
            ("r2", r2),
            ("r3", r3),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        self.parse_section_flags(output, flags);
        return output;
    }
    fn parse_segment_flags<T0, RT>(&self, flags: T0) -> RT {
        "Parse segment flags into readable list.";
        let mut output = vec![];
        let mut i = 1;
        while i < 9 {
            if (flags & i) == i {
                output.push(mdictionary::segment_flags[i]);
            }
            i <<= 1;
        }
        return output;
    }
    fn parse_segment<T0, T1, T2, T3, RT>(
        &self,
        m_offset: T0,
        m_size: T1,
        cmd: T2,
        cmd_size: T3,
    ) -> RT {
        "Parse segment command.";
        let name = self.__file.read(16).decode().rstrip(" ");
        let vmaddr = if self.__is_64_bit {
            self.get_ll()
        } else {
            self.get_int()
        };
        let vmsize = if self.__is_64_bit {
            self.get_ll()
        } else {
            self.get_int()
        };
        let offset = if self.__is_64_bit {
            self.get_ll()
        } else {
            self.get_int()
        };
        let segsize = if self.__is_64_bit {
            self.get_ll()
        } else {
            self.get_int()
        };
        let mut maxprot = self.get_int();
        let mut initprot = self.get_int();
        let nsects = self.get_int();
        let flags = self.get_int();
        maxprot = mdictionary::protections[(maxprot & 7)];
        initprot = mdictionary::protections[(initprot & 7)];
        let entropy = self.get_segment_entropy(m_offset, offset, segsize);
        let output = [
            ("m_offset", m_offset),
            ("cmd", cmd),
            ("size", cmd_size),
            ("name", name),
            ("vmaddr", vmaddr),
            ("vmsize", vmsize),
            ("offset", offset),
            ("segsize", segsize),
            ("maxprot", maxprot),
            ("initprot", initprot),
            ("nsects", nsects),
            ("entropy", entropy),
            ("sects", vec![]),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        let sect_size = if self.__is_64_bit { 80 } else { 68 };
        for _ in (0..nsects) {
            if (self.__file.tell() + sect_size) > (m_offset + m_size) {
                self.add_abnormality(
                    "Section at offset \"{}\" with size \"{}\" greater than mach-o size."
                        .format(self.__file.tell(), sect_size),
                );
                break;
            }
            output["sects"].append(self.parse_section());
        }
        output["flags"] = self.parse_segment_flags(flags);
        return output;
    }
    fn parse_symtab<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse symbol table load command.";
        let symoff = self.get_int();
        let nsyms = self.get_int();
        let stroff = self.get_int();
        let strsize = self.get_int();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("symoff", symoff),
            ("nsyms", nsyms),
            ("stroff", stroff),
            ("strsize", strsize),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_symseg<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse link-edit gdb symbol table info (obsolete).";
        let offset = self.get_int();
        let size = self.get_int();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("offset", offset),
            ("size", size),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_thread<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse thread load command.";
        let mut state = self.get_int();
        let count = self.get_int();
        self.__file.read((cmd_size - 16));
        if mdictionary::thread_states.iter().any(|&x| x == state) {
            state = mdictionary::thread_states[state];
        } else {
            self.add_abnormality(
                "Invalid THREAD STATE FLAVOR \"{}\" at offset \"{}\"."
                    .format(state, (self.__file.tell() - 8)),
            );
        }
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("state", state),
            ("count", count),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_fvmlib<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse fvmlib load command.";
        let offset = (self.__file.tell() - 8);
        self.__file.read(4);
        let minor_version = self.get_int();
        let header_addr = self.get_int();
        let name = self.get_string();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("name", name),
            ("minor_version", self.make_version(minor_version)),
            ("header_addr", header_addr),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        self.__file.read((cmd_size - (self.__file.tell() - offset)));
        return output;
    }
    fn parse_ident<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse object identification info (obsolete).";
        let output = [("cmd", cmd), ("cmd_size", cmd_size), ("strings", vec![])]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        let end = ((self.__file.tell() - 8) + cmd_size);
        while self.__file.tell() < end {
            let string = self.get_string();
            if string != "" {
                output["strings"].append(string);
            }
        }
        return output;
    }
    fn parse_fvmfile<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse fixed VM file inclusion (internal use).";
        let name = self.get_string();
        let header_addr = self.get_int();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("name", name),
            ("header_addr", header_addr),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_prepage<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse prepage command (internal use). Load command structure not
        found.
        ";
        self.__file.read((cmd_size - 8));
        let output = [("cmd", cmd), ("cmd_size", cmd_size)]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_dysymtab<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse dynamic link-edit symbol table info.";
        let ilocalsym = self.get_int();
        let nlocalsym = self.get_int();
        let iextdefsym = self.get_int();
        let nextdefsym = self.get_int();
        let iundefsym = self.get_int();
        let nundefsym = self.get_int();
        let tocoff = self.get_int();
        let ntoc = self.get_int();
        let modtaboff = self.get_int();
        let nmodtab = self.get_int();
        let extrefsymoff = self.get_int();
        let nextrefsyms = self.get_int();
        let indirectsymoff = self.get_int();
        let nindirectsyms = self.get_int();
        let extreloff = self.get_int();
        let nextrel = self.get_int();
        let locreloff = self.get_int();
        let nlocrel = self.get_int();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("ilocalsym", ilocalsym),
            ("nlocalsym", nlocalsym),
            ("iextdefsym", iextdefsym),
            ("nextdefsym", nextdefsym),
            ("iundefsym", iundefsym),
            ("nundefsym", nundefsym),
            ("tocoff", tocoff),
            ("ntoc", ntoc),
            ("modtaboff", modtaboff),
            ("nmodtab", nmodtab),
            ("extrefsymoff", extrefsymoff),
            ("nextrefsyms", nextrefsyms),
            ("indirectsymoff", indirectsymoff),
            ("nindirectsyms", nindirectsyms),
            ("extreloff", extreloff),
            ("nextrel", nextrel),
            ("locreloff", locreloff),
            ("nlocrel", nlocrel),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_load_dylib<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse dylib load command.";
        let offset = (self.__file.tell() - 8);
        self.__file.read(4);
        let timestamp = self.get_int();
        let current_version = self.get_int();
        let compatibility_version = self.get_int();
        let name = self.get_string();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("name", name),
            (
                "timestamp",
                datetime::fromtimestamp(timestamp).strftime("%Y-%m-%d %H:%M:%S"),
            ),
            ("current_version", self.make_version(current_version)),
            (
                "compatability_version",
                self.make_version(compatibility_version),
            ),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        self.__file.read((cmd_size - (self.__file.tell() - offset)));
        return output;
    }
    fn parse_load_dylinker<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse dylinker load command.";
        let offset = (self.__file.tell() - 8);
        self.__file.read(4);
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("name", self.get_string()),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        self.__file.read((cmd_size - (self.__file.tell() - offset)));
        return output;
    }
    fn parse_prebound_dylib<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse prebound dylib load command.  An executable that is prebound to
        its dynamic libraries will have one of these for each library that the
        static linker used in prebinding.
        ";
        let name = self.get_string();
        let nmodules = self.get_int();
        let linked_modules = self.get_string();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("name", name),
            ("nmodules", nmodules),
            ("linked_modules", linked_modules),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_routines<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse routines load command. The routines command contains the
        address of the dynamic shared library initialization routine and an
        index into the module table for the module that defines the routine.
        ";
        let init_address = if self.__is_64_bit {
            self.get_ll()
        } else {
            self.get_int()
        };
        let init_module = if self.__is_64_bit {
            self.get_ll()
        } else {
            self.get_int()
        };
        if self.__is_64_bit {
            self.__file.read(48)
        } else {
            self.__file.read(24)
        };
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("init_address", init_address),
            ("init_module", init_module),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_sub_stuff<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse sub_* load command.";
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("name", self.get_string()),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_twolevel_hints<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse two-level hints load command.";
        let offset = self.get_int();
        let nhints = self.get_int();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("offset", offset),
            ("nhints", nhints),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_prebind_cksum<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse prebind checksum load command.";
        let cksum = self.get_int();
        let output = [("cmd", cmd), ("cmd_size", cmd_size), ("cksum", cksum)]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_uuid<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse UUID load command.";
        let mut uuid = self.__file.read(16);
        if self.__is_little_endian {
            uuid = unpack("<16s", uuid)[0];
        }
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("uuid", UUID(uuid).hex),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_linkedit_data<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse link-edit data load command.";
        let dataoff = self.get_int();
        let datasize = self.get_int();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("dataoff", dataoff),
            ("datasize", datasize),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_encryption_info<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse encryption info load command. Contains the file offset and size
        of an encrypted segment.
        ";
        let cryptoff = self.get_int();
        let cryptsize = self.get_int();
        let cryptid = self.get_int();
        if cmd.endswith("64") {
            self.__file.read(4);
        }
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("cryptoff", cryptoff),
            ("cryptsize", cryptsize),
            ("cryptid", cryptid),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_dyld_info<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse dyld info load command. contains the file offsets and sizes of
        the new compressed form of the information dyld needs to load the
        image. This information is used by dyld on Mac OS X 10.6 and later. All
        information pointed to by this command is encoded using byte streams,
        so no endian swapping is needed to interpret it.
        ";
        let rebase_off = self.get_int();
        let rebase_size = self.get_int();
        let bind_off = self.get_int();
        let bind_size = self.get_int();
        let weak_bind_off = self.get_int();
        let weak_bind_size = self.get_int();
        let lazy_bind_off = self.get_int();
        let lazy_bind_size = self.get_int();
        let export_off = self.get_int();
        let export_size = self.get_int();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("rebase_off", rebase_off),
            ("rebase_size", rebase_size),
            ("bind_off", bind_off),
            ("bind_size", bind_size),
            ("weak_bind_off", weak_bind_off),
            ("weak_bind_size", weak_bind_size),
            ("lazy_bind_off", lazy_bind_off),
            ("lazy_bind_size", lazy_bind_size),
            ("export_off", export_off),
            ("export_size", export_size),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_version_min_os<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse minimum OS version load command.";
        let version = self.get_int();
        let sdk = self.get_int();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("version", self.make_version(version)),
            ("sdk", self.make_version(sdk)),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_source_version<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse source version load command.";
        let version = self.get_ll();
        let mask = 1023;
        let a = (version >> 40);
        let b = ((version >> 30) & mask);
        let c = ((version >> 20) & mask);
        let d = ((version >> 10) & mask);
        let e = (version & mask);
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("version", "{}.{}.{}.{}.{}".format(a, b, c, d, e)),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_linker_option<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse linker options load command.";
        let start = (self.__file.tell() - 8);
        let count = self.get_int();
        let mut linker_options = vec![];
        for _ in (0..count) {
            linker_options.push(self.get_string());
        }
        self.__file.read((cmd_size - (self.__file.tell() - start)));
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("count", count),
            ("linker_options", linker_options),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_rpath<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse rpath load command.";
        let offset = (self.__file.tell() - 8);
        self.__file.read(4);
        let path = self.get_string();
        let output = [("cmd", cmd), ("cmd_size", cmd_size), ("path", path)]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        self.__file.read((cmd_size - (self.__file.tell() - offset)));
        return output;
    }
    fn parse_main<T0, T1, RT>(&self, cmd: T0, cmd_size: T1) -> RT {
        "Parse main load command.";
        let entryoff = self.get_ll();
        let stacksize = self.get_ll();
        let output = [
            ("cmd", cmd),
            ("cmd_size", cmd_size),
            ("entryoff", entryoff),
            ("stacksize", stacksize),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        return output;
    }
    fn parse_lcs<T0, T1, T2, T3>(&self, offset: T0, size: T1, nlcs: T2, slcs: T3) {
        "Determine which load commands are present and parse each one
        accordingly. Return as a list.

        Load command structures found in '/usr/include/mach-o/loader.h'.
        ";
        self.__macho["lcs"] = vec![];
        self.segments = vec![];
        for _ in (0..nlcs) {
            let mut cmd = self.get_int();
            let cmd_size = self.get_int();
            if self.__is_64_bit && (cmd_size % 8) != 0 {
                raise!(ValueError("Load command size \"{}\" for 64-bit mach-o at offset \"{}\" is not divisible by 8.".format(cmd_size, (self.__file.tell() - 4))));
                //unsupported
            } else {
                if (cmd_size % 4) != 0 {
                    raise!(ValueError("Load command size \"{}\" for 32-bit mach-o at offset \"{}\" is not divisible by 4.".format(cmd_size, (self.__file.tell() - 4))));
                    //unsupported
                }
            }
            if mdictionary::loadcommands.iter().any(|&x| x == cmd) {
                cmd = mdictionary::loadcommands[cmd];
            } else {
                self.add_abnormality(
                    "Unknown load command \"{}\" at offset \"{}\"."
                        .format(cmd, (self.__file.tell() - 8)),
                );
                self.__file.read((cmd_size - 8));
            }
            if cmd == "SEGMENT" || cmd == "SEGMENT_64" {
                let parsed = self.parse_segment(offset, size, cmd, cmd_size);
                self.__macho["lcs"].append(parsed);
                self.segments.append(parsed);
            } else {
                if cmd == "SYMTAB" {
                    self.symtab = self.parse_symtab(cmd, cmd_size);
                    self.__macho["lcs"].append(self.symtab);
                } else {
                    if cmd == "SYMSEG" {
                        self.__macho["lcs"].append(self.parse_symseg(cmd, cmd_size));
                    } else {
                        if ("THREAD", "UNIXTHREAD").iter().any(|&x| x == cmd) {
                            self.__macho["lcs"].append(self.parse_thread(cmd, cmd_size));
                        } else {
                            if ("LOADFVMLIB", "IDFVMLIB").iter().any(|&x| x == cmd) {
                                self.__macho["lcs"].append(self.parse_fvmlib(cmd, cmd_size));
                            } else {
                                if cmd == "IDENT" {
                                    self.__macho["lcs"].append(self.parse_ident(cmd, cmd_size));
                                } else {
                                    if cmd == "FVMFILE" {
                                        self.__macho["lcs"]
                                            .append(self.parse_fvmfile(cmd, cmd_size));
                                    } else {
                                        if cmd == "PREPAGE" {
                                            self.__macho["lcs"]
                                                .append(self.parse_prepage(cmd, cmd_size));
                                        } else {
                                            if cmd == "DYSYMTAB" {
                                                self.__macho["lcs"]
                                                    .append(self.parse_dysymtab(cmd, cmd_size));
                                            } else {
                                                if (
                                                    "LOAD_DYLIB",
                                                    "ID_DYLIB",
                                                    "LAZY_LOAD_DYLIB",
                                                    "LOAD_WEAK_DYLIB",
                                                    "REEXPORT_DYLIB",
                                                    "LOAD_UPWARD_DYLIB",
                                                )
                                                    .iter()
                                                    .any(|&x| x == cmd)
                                                {
                                                    self.__macho["lcs"].append(
                                                        self.parse_load_dylib(cmd, cmd_size),
                                                    );
                                                } else {
                                                    if (
                                                        "LOAD_DYLINKER",
                                                        "ID_DYLINKER",
                                                        "DYLD_ENVIRONMENT",
                                                    )
                                                        .iter()
                                                        .any(|&x| x == cmd)
                                                    {
                                                        self.__macho["lcs"].append(
                                                            self.parse_load_dylinker(cmd, cmd_size),
                                                        );
                                                    } else {
                                                        if cmd == "PREBOUND_DYLIB" {
                                                            self.__macho["lcs"].append(
                                                                self.parse_prebound_dylib(
                                                                    cmd, cmd_size,
                                                                ),
                                                            );
                                                        } else {
                                                            if ("ROUTINES", "ROUTINES_64")
                                                                .iter()
                                                                .any(|&x| x == cmd)
                                                            {
                                                                self.__macho["lcs"].append(
                                                                    self.parse_routines(
                                                                        cmd, cmd_size,
                                                                    ),
                                                                );
                                                            } else {
                                                                if (
                                                                    "SUB_FRAMEWORK",
                                                                    "SUB_UMBRELLA",
                                                                    "SUB_CLIENT",
                                                                    "SUB_LIBRARY",
                                                                )
                                                                    .iter()
                                                                    .any(|&x| x == cmd)
                                                                {
                                                                    self.__macho["lcs"].append(
                                                                        self.parse_sub_stuff(
                                                                            cmd, cmd_size,
                                                                        ),
                                                                    );
                                                                } else {
                                                                    if cmd == "TWOLEVEL_HINTS" {
                                                                        self.__macho["lcs"].append(self.parse_twolevel_hints(cmd, cmd_size));
                                                                    } else {
                                                                        if cmd == "PREBIND_CKSUM" {
                                                                            self.__macho["lcs"].append(self.parse_prebind_cksum(cmd, cmd_size));
                                                                        } else {
                                                                            if cmd == "UUID" {
                                                                                self.__macho["lcs"].append(self.parse_uuid(cmd, cmd_size));
                                                                            } else {
                                                                                if ("CODE_SIGNATURE", "SEGMENT_SPLIT_INFO", "FUNCTION_STARTS", "DATA_IN_CODE", "DYLIB_CODE_SIGN_DRS", "LINKER_OPTIMIZATION_HINT").iter().any(|&x| x == cmd) {
self.__macho["lcs"].append(self.parse_linkedit_data(cmd, cmd_size));
} else {
if ("ENCRYPTION_INFO", "ENCRYPTION_INFO_64").iter().any(|&x| x == cmd) {
self.__macho["lcs"].append(self.parse_encryption_info(cmd, cmd_size));
} else {
if ("DYLD_INFO", "DYLD_INFO_ONLY").iter().any(|&x| x == cmd) {
self.dyld_info = self.parse_dyld_info(cmd, cmd_size);
self.__macho["lcs"].append(self.dyld_info);
} else {
if ("VERSION_MIN_MACOSX", "VERSION_MIN_IPHONEOS", "VERSION_MIN_WATCHOS", "VERSION_MIN_TVOS").iter().any(|&x| x == cmd) {
self.__macho["lcs"].append(self.parse_version_min_os(cmd, cmd_size));
} else {
if cmd == "SOURCE_VERSION" {
self.__macho["lcs"].append(self.parse_source_version(cmd, cmd_size));
} else {
if cmd == "LINKER_OPTION" {
self.__macho["lcs"].append(self.parse_linker_option(cmd, cmd_size));
} else {
if cmd == "RPATH" {
self.__macho["lcs"].append(self.parse_rpath(cmd, cmd_size));
} else {
if cmd == "MAIN" {
self.__macho["lcs"].append(self.parse_main(cmd, cmd_size));
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
    fn parse_syms<T0, T1, T2>(&self, offset: T0, size: T1, lc_symtab: T2) {
        "Parse symbol and string tables.

        Symbol table format found in:
        /usr/include/mach-o/nlist.h
        /usr/include/mach-o/stab.h
        ";
        if lc_symtab["symoff"] > size {
            self.add_abnormality(
                "Symbol table at offset \"{}\" out of bounds."
                    .format((offset + lc_symtab["symoff"])),
            );
            return;
        }
        let true_offset = (offset + lc_symtab["symoff"]);
        let symbol_size = if self.__is_64_bit { 16 } else { 12 };
        self.__file.seek(true_offset);
        let entropy = self.calc_entropy(self.__file.read((lc_symtab["nsyms"] * symbol_size)));
        if entropy >= 0.8 {
            self.add_abnormality(
                "Symbol table with entropy of \"{}\" is probably packed. Not attempting to parse."
                    .format(entropy),
            );
            return;
        }
        if (lc_symtab["symoff"] + (lc_symtab["nsyms"] * symbol_size)) > size {
            self.add_abnormality("Symbol table at offset \"{}\" partially out of bounds. Attempting to parse as many symbols as possible.".format(true_offset));
        }
        self.__file.seek(true_offset);
        self.__macho["symtab"] = vec![];
        for _ in (0..lc_symtab["nsyms"]) {
            if (self.__file.tell() + symbol_size) > (offset + size) {
                break;
            }
            let n_strx = self.get_int();
            let mut n_type = i32::from(self.__file.read(1).hex(), 16);
            let n_sect = i32::from(self.__file.read(1).hex(), 16);
            let n_desc = i32::from(self.__file.read(2).hex(), 16);
            let n_value = if self.__is_64_bit {
                self.get_ll()
            } else {
                self.get_int()
            };
            let symbol = [
                ("n_strx", n_strx),
                ("n_sect", n_sect),
                ("n_desc", n_desc),
                ("n_value", n_value),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
            if n_type >= 32 {
                if mdictionary::stabs.iter().any(|&x| x == n_type) {
                    symbol["stab"] = mdictionary::stabs[n_type];
                } else {
                    self.add_abnormality(
                        "Unknown stab type \"{}\" at offset \"{}\"."
                            .format(n_type, ((self.__file.tell() - symbol_size) + 4)),
                    );
                }
            } else {
                let n_pext = (n_type & 16);
                let n_ext = (n_type & 1);
                n_type = (n_type & 14);
                if mdictionary::n_types.iter().any(|&x| x == n_type) {
                    n_type = mdictionary::n_types[n_type];
                } else {
                    self.add_abnormality(
                        "Unknown N_TYPE \"{}\" at offset \"{}\"."
                            .format(n_type, ((self.__file.tell() - symbol_size) + 4)),
                    );
                }
                if self.__is_little_endian {
                    dylib = (n_desc & 15);
                    let mut ref_ = ((n_desc >> 8) & 255);
                } else {
                    dylib = ((n_desc >> 8) & 255);
                    ref_ = (n_desc & 15);
                }
                symbol["pext"] = n_pext;
                symbol["n_type"] = n_type;
                symbol["ext"] = n_ext;
                symbol["dylib"] = dylib;
                symbol["ref"] = ref_;
            }
            self.__macho["symtab"].append(symbol);
        }
    }
    fn parse_strings<T0, T1, T2>(&self, offset: T0, size: T1, lc_symtab: T2) {
        "Parse string table.";
        if lc_symtab["stroff"] > size {
            self.add_abnormality(
                "String table at offset \"{}\" greater than mach-o size."
                    .format((offset + lc_symtab["stroff"])),
            );
            return;
        }
        let true_offset = (offset + lc_symtab["stroff"]);
        self.__file.seek(true_offset);
        let entropy = self.calc_entropy(self.__file.read(lc_symtab["strsize"]));
        if entropy >= 0.8 {
            self.add_abnormality(
                "String table with entropy of \"{}\" is probably packed. Not attempting to parse."
                    .format(entropy),
            );
            return;
        }
        if (true_offset + lc_symtab["strsize"]) > (offset + size) {
            self.add_abnormality("String Table at offset \"{}\" partially out of bounds. Attempting to parse as many strings as possible.".format(true_offset));
        }
        self.__macho["strtab"] = vec![];
        self.__file.seek(true_offset);
        while self.__file.tell() < (true_offset + lc_symtab["strsize"]) {
            let try_dummy = {
                //unsupported
                let string = self.get_string();
                if string != "" {
                    self.__macho["strtab"].append(string);
                }
            };
            let except!() = {
                //unsupported
                break;
            };
        }
    }
    fn parse_imports<T0, T1, T2, T3, T4>(
        &self,
        offset: T0,
        size: T1,
        lc_symtab: T2,
        lc_dysymtab: T3,
        lc_dylibs: T4,
    ) {
        "Parse undefined external symbols (imports) out of the symbol and
        string tables.
        ";
        self.__macho["imports"] = vec![];
        let true_offset = (offset + lc_symtab["stroff"]);
        let mut undef_syms = None;
        if lc_dysymtab != None {
            let i_undef = ((lc_dysymtab["nlocalsym"] + lc_dysymtab["nextdefsym"]) - 1);
            let j_undef = (i_undef + lc_dysymtab["nundefsym"]);
            undef_syms = self.__macho["symtab"][i_undef..j_undef];
        } else {
            undef_syms = self.__macho["symtab"]
                .into_iter()
                .filter(|sym| ("UNDF", "PBUD").iter().any(|&x| x == sym["n_type"]));
        }
        for sym in undef_syms {
            self.__file.seek((true_offset + sym["n_strx"]));
            let value = self.get_string();
            if lc_dylibs != None {
                let mut dylib = sym["dylib"];
                if dylib == 0 {
                    dylib = "SELF_LIBRARY";
                } else {
                    if dylib == 254 {
                        dylib = "DYNAMIC_LOOKUP";
                    } else {
                        if dylib == 255 {
                            dylib = "EXECUTABLE";
                        } else {
                            if dylib > lc_dylibs.len() {
                                dylib = None;
                            } else {
                                dylib = lc_dylibs[(dylib - 1)]["name"];
                            }
                        }
                    }
                }
                self.__macho["imports"].append((value, dylib));
            } else {
                self.__macho["imports"].append(value);
            }
        }
    }
    fn parse_certs<T0, T1, RT>(&self, sig_offset: T0, index_offset: T1) -> RT {
        "Parse X509 certificates out of code signature.";
        let prev = self.__file.tell();
        let true_offset = (sig_offset + index_offset);
        self.__file.seek(true_offset);
        let magic = self.get_int(true);
        if magic != mdictionary::signatures["BLOBWRAPPER"] {
            self.add_abnormality(
                "Bad magic \"{}\" for certificate blob wrapper at offset \"{}\"."
                    .format(magic, true_offset),
            );
            return vec![];
        }
        let size = (self.get_int(true) - 8);
        if size <= 0 {
            self.add_abnormality(
                "Non-positive CMS size \"{}\" at offset \"{}\"."
                    .format(size, (self.__file.tell() - 4)),
            );
            return vec![];
        }
        let signed_data = ContentInfo.load(self.__file.read(size))["content"];
        self.__macho["code_signature"]["certs"] = vec![];
        for cert in signed_data["certificates"] {
            cert = cert.chosen;
            if self.__extract_certs {
                let c_bytes = cert.dump();
                open(hashlib.md5(c_bytes).hexdigest(), "wb").write(c_bytes);
            }
            let subject = HashMap::new();
            for rdn in cert.subject.chosen {
                let mut name = rdn[0]["type"].human_friendly;
                let mut value = rdn[0]["value"];
                if name == "Country" {
                    subject["country"] = String::from(value.chosen);
                } else {
                    if name == "Organization" {
                        subject["org"] = String::from(value.chosen);
                    } else {
                        if name == "Organizational Unit" {
                            subject["org_unit"] = String::from(value.chosen);
                        } else {
                            if name == "Common Name" {
                                subject["common_name"] = String::from(value.chosen);
                            } else {
                                if isinstance(value, DirectoryString) {
                                    subject[name] = String::from(value.chosen);
                                } else {
                                    subject[name] = String::from(value.parsed);
                                }
                            }
                        }
                    }
                }
            }
            let issuer = HashMap::new();
            for rdn in cert.issuer.chosen {
                let mut name = rdn[0]["type"].human_friendly;
                let mut value = rdn[0]["value"];
                if name == "Country" {
                    issuer["country"] = String::from(value.chosen);
                } else {
                    if name == "Organization" {
                        issuer["org"] = String::from(value.chosen);
                    } else {
                        if name == "Organizational Unit" {
                            issuer["org_unit"] = String::from(value.chosen);
                        } else {
                            if name == "Common Name" {
                                issuer["common_name"] = String::from(value.chosen);
                            } else {
                                if isinstance(value, DirectoryString) {
                                    issuer[name] = String::from(value.chosen);
                                } else {
                                    issuer[name] = String::from(value.parsed);
                                }
                            }
                        }
                    }
                }
            }
            let certificate = [
                ("subject", subject),
                ("issuer", issuer),
                ("serial", cert.serial_number),
                ("is_ca", cert.ca),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
            self.__macho["code_signature"]["certs"].append(certificate);
        }
        self.__file.seek(prev);
    }
    fn parse_codedirectory<T0, T1>(&self, sig_offset: T0, index_offset: T1) {
        "Parse code directory from code signature.";
        let prev = self.__file.tell();
        let true_offset = (sig_offset + index_offset);
        self.__file.seek(true_offset);
        let magic = self.get_int(true);
        if magic != mdictionary::signatures["CODEDIRECTORY"] {
            self.add_abnormality(
                "Bad magic \"{}\" for code directory at offset \"{}\"."
                    .format(magic, (self.__file.tell() - 4)),
            );
            return;
        }
        let size = self.get_int(true);
        let version = self.get_int(true);
        let flags = self.get_int(true);
        let hash_offset = self.get_int(true);
        let ident_offset = self.get_int(true);
        let n_special_slots = self.get_int(true);
        let n_code_slots = self.get_int(true);
        let code_limit = self.get_int(true);
        let hash_size = i32::from(self.__file.read(1).hex(), 16);
        let hash_type = mdictionary::hashes[i32::from(self.__file.read(1).hex(), 16)];
        if version >= 131584 {
            let platform = i32::from(self.__file.read(1).hex(), 16);
        } else {
            self.__file.read(1);
        }
        let page_size = i32::from(round(exp(
            (i32::from(self.__file.read(1).hex(), 16) * log(2))
        )));
        self.__file.read(4);
        if version >= 131328 {
            let scatter_offset = self.get_int(true);
        }
        if version >= 131584 {
            let team_id_offset = self.get_int(true);
            self.__file.seek((true_offset + team_id_offset));
            let team_id = self.get_string();
        }
        self.__file.seek((true_offset + ident_offset));
        let identity = self.get_string();
        self.__macho["code_signature"]["codedirectory"] = [
            ("size", size),
            ("version", version),
            ("flags", flags),
            ("hash_offset", hash_offset),
            ("n_special_slots", n_special_slots),
            ("n_code_slots", n_code_slots),
            ("code_limit", code_limit),
            ("hash_size", hash_size),
            ("hash_type", hash_type),
            ("page_size", page_size),
            ("identity", identity),
            ("hashes", vec![]),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        if version >= 131328 {
            self.__macho["code_signature"]["codedirectory"]["scatter_offset"] = scatter_offset;
        }
        if version >= 131584 {
            self.__macho["code_signature"]["codedirectory"]["platform"] = platform;
            self.__macho["code_signature"]["codedirectory"]["team_id_offset"] = team_id_offset;
            self.__macho["code_signature"]["codedirectory"]["team_id"] = team_id;
        }
        self.__file
            .seek(((true_offset + hash_offset) - (n_special_slots * hash_size)));
        let count = (n_special_slots + n_code_slots);
        for _ in (0..count) {
            self.__macho["code_signature"]["codedirectory"]["hashes"]
                .append(self.__file.read(hash_size).hex());
        }
        self.__file.seek(prev);
    }
    fn get_oid<T0, T1, RT>(&self, db: T0, p: T1) -> RT {
        "OID parser implementation from:

        http://opensource.apple.com/source/Security/Security-57337.20.44/
        OSX/libsecurity_cdsa_utilities/lib/cssmdata.cpp
        ";
        let mut q = 0;
        while true {
            q = ((q * 128) + (db[p] & None128));
            if p < db.len() && (db[p] & 128) {
                p += 1;
            } else {
                p += 1;
                break;
            }
        }
        return (q, p);
    }
    fn to_oid<T0, RT>(&self, length: T0) -> RT {
        "Convert bytes to correct OID.";
        if length == 0 {
            return "";
        }
        let data_bytes = (0..length)
            .iter()
            .map(|i| i32::from(self.__file.read(1).hex(), 16))
            .collect::<Vec<_>>();
        let p = 0;
        let (oid1, p) = self.get_oid(data_bytes, p);
        let q1 = (oid1 / 40).iter().min().unwrap();
        let mut data = ((String::from(q1) + ".") + String::from((oid1 - (q1 * 40))));
        while p < data_bytes.len() {
            let (d, p) = self.get_oid(data_bytes, p);
            data += ("." + String::from(d));
        }
        self.__file.read((-(length) & 3));
        return data;
    }
    fn parse_entitlement<T0, T1>(&self, sig_offset: T0, index_offset: T1) {
        "Parse entitlement from code signature.";
        let prev = self.__file.tell();
        let true_offset = (sig_offset + index_offset);
        self.__file.seek(true_offset);
        let magic = self.get_int(true);
        if magic != mdictionary::signatures["ENTITLEMENT"] {
            self.add_abnormality(
                "Bad magic \"{}\" for entitlement at offset \"{}\"."
                    .format(magic, (self.__file.tell() - 4)),
            );
            return;
        }
        let size = (self.get_int(true) - 8);
        let try_dummy = {
            //unsupported
            let mut plist = loads(self.__file.read(size));
        };
        let except!(Exception) = {
            //unsupported
            plist = HashMap::new();
            self.add_abnormality(
                "Unable to parse plist at offset \"{}\". {}."
                    .format((self.__file.tell() - size), exc),
            );
        };
        if self.__macho["code_signature"]
            .iter()
            .all(|&x| x != "entitlements")
        {
            self.__macho["code_signature"]["entitlements"] = vec![];
        }
        self.__macho["code_signature"]["entitlements"].append(
            [("size", size), ("plist", plist)]
                .iter()
                .cloned()
                .collect::<HashMap<_, _>>(),
        );
        self.__file.seek(prev);
    }
    fn parse_data<RT>(&self) -> RT {
        "Parse data for requirement expression.";
        let length = self.get_int(true);
        let data = self.__file.read(length);
        self.__file.read((-(length) & 3));
        return data;
    }
    fn parse_match<RT>(&self) -> RT {
        "Parse match for requirement expression.";
        let mut match_type = self.get_int(true);
        if mdictionary::matches.iter().any(|&x| x == match_type) {
            match_type = mdictionary::matches[match_type];
        }
        if match_type == "matchExists" {
            return " /* exists */";
        } else {
            if match_type == "matchEqual" {
                return " = \"{}\"".format(self.parse_data().decode());
            } else {
                if match_type == "matchContains" {
                    return " ~ \"{}\"".format(self.parse_data().decode());
                } else {
                    if match_type == "matchBeginsWith" {
                        return " = \"{}*\"".format(self.parse_data().decode());
                    } else {
                        if match_type == "matchEndsWith" {
                            return " = \"*{}\"".format(self.parse_data().decode());
                        } else {
                            if match_type == "matchLessThan" {
                                return " < {}".format(i32::from(self.parse_data(), 16));
                            } else {
                                if match_type == "matchGreaterThan" {
                                    return " > {}".format(i32::from(self.parse_data(), 16));
                                } else {
                                    if match_type == "matchLessEqual" {
                                        return " <= {}".format(i32::from(self.parse_data(), 16));
                                    } else {
                                        if match_type == "matchGreaterEqual" {
                                            return " >= {}"
                                                .format(i32::from(self.parse_data(), 16));
                                        } else {
                                            return " UNKNOWN MATCH TYPE \"{}\"".format(match_type);
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
    fn parse_expression<T0, RT>(&self, in_or: T0) -> RT {
        "Parse requirement expression. Recurse if necessary";
        let mut operator = self.get_int(true);
        operator = mdictionary::operators[(operator & 4095)];
        let mut expression = "";
        if operator == "False" {
            expression += "never";
        } else {
            if operator == "True" {
                expression += "always";
            } else {
                if operator == "Ident" {
                    expression += "identity \"{}\"".format(self.parse_data().decode());
                } else {
                    if operator == "AppleAnchor" {
                        expression += "anchor apple";
                    } else {
                        if operator == "AppleGenericAnchor" {
                            expression += "anchor apple generic";
                        } else {
                            if operator == "AnchorHash" {
                                let mut cert_slot = self.get_int(true);
                                if mdictionary::cert_slots.iter().any(|&x| x == cert_slot) {
                                    cert_slot = mdictionary::cert_slots[cert_slot];
                                }
                                expression += "certificate {} = {}"
                                    .format(cert_slot, self.parse_data().decode());
                            } else {
                                if operator == "InfoKeyValue" {
                                    expression += "info[{}] = \"{}\"".format(
                                        self.parse_data().decode(),
                                        self.parse_data().decode(),
                                    );
                                } else {
                                    if operator == "And" {
                                        if in_or {
                                            expression += "({} and {})".format(
                                                self.parse_expression(),
                                                self.parse_expression(),
                                            );
                                        } else {
                                            expression += "{} and {}".format(
                                                self.parse_expression(),
                                                self.parse_expression(),
                                            );
                                        }
                                    } else {
                                        if operator == "Or" {
                                            if in_or {
                                                expression += "({} or {})".format(
                                                    self.parse_expression(true),
                                                    self.parse_expression(true),
                                                );
                                            } else {
                                                expression += "{} or {}".format(
                                                    self.parse_expression(true),
                                                    self.parse_expression(true),
                                                );
                                            }
                                        } else {
                                            if operator == "Not" {
                                                expression +=
                                                    "! {}".format(self.parse_expression());
                                            } else {
                                                if operator == "CDHash" {
                                                    expression += "cdhash {}"
                                                        .format(self.parse_data().decode());
                                                } else {
                                                    if operator == "InfoKeyField" {
                                                        expression += "info[{}]{}".format(
                                                            self.parse_data().decode(),
                                                            self.parse_match(),
                                                        );
                                                    } else {
                                                        if operator == "EntitlementField" {
                                                            expression += "entitlement[{}]{}"
                                                                .format(
                                                                    self.parse_data().decode(),
                                                                    self.parse_match(),
                                                                );
                                                        } else {
                                                            if operator == "CertField" {
                                                                cert_slot = self.get_int(true);
                                                                if mdictionary::cert_slots
                                                                    .iter()
                                                                    .any(|&x| x == cert_slot)
                                                                {
                                                                    cert_slot =
                                                                        mdictionary::cert_slots
                                                                            [cert_slot];
                                                                }
                                                                expression +=
                                                                    "certificate {}[{}]{}".format(
                                                                        cert_slot,
                                                                        self.parse_data().decode(),
                                                                        self.parse_match(),
                                                                    );
                                                            } else {
                                                                if operator == "CertGeneric" {
                                                                    cert_slot = self.get_int(true);
                                                                    if mdictionary::cert_slots
                                                                        .iter()
                                                                        .any(|&x| x == cert_slot)
                                                                    {
                                                                        cert_slot =
                                                                            mdictionary::cert_slots
                                                                                [cert_slot];
                                                                    }
                                                                    let length = self.get_int(true);
                                                                    expression += "certificate {}[field.{}]{}".format(cert_slot, self.to_oid(length), self.parse_match());
                                                                } else {
                                                                    if operator == "CertPolicy" {
                                                                        cert_slot =
                                                                            self.get_int(true);
                                                                        if mdictionary::cert_slots
                                                                            .iter()
                                                                            .any(|&x| {
                                                                                x == cert_slot
                                                                            })
                                                                        {
                                                                            cert_slot = mdictionary::cert_slots[cert_slot];
                                                                        }
                                                                        expression += "certificate {}[policy.{}]{}".format(cert_slot, self.parse_data().decode(), self.parse_match());
                                                                    } else {
                                                                        if operator == "TrustedCert"
                                                                        {
                                                                            cert_slot =
                                                                                self.get_int(true);
                                                                            if mdictionary::cert_slots.iter().any(|&x| x == cert_slot) {
cert_slot = mdictionary::cert_slots[cert_slot];
}
                                                                            expression += "certificate {} trusted".format(cert_slot);
                                                                        } else {
                                                                            if operator
                                                                                == "TrustedCerts"
                                                                            {
                                                                                expression += "anchor trusted";
                                                                            } else {
                                                                                if operator
                                                                                    == "NamedAnchor"
                                                                                {
                                                                                    expression += "anchor apple {}".format(self.parse_data().decode());
                                                                                } else {
                                                                                    if operator == "NamedCode" {
expression += "({})".format(self.parse_data().decode());
} else {
if operator == "Platform" {
let platform = self.get_int(true);
expression += "platform = {}".format(platform);
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
                        }
                    }
                }
            }
        }
        return expression;
    }
    fn parse_requirement<T0, T1, T2>(&self, reqs_offset: T0, req_type: T1, req_offset: T2) {
        "Parse single requirement from code signature.";
        let prev = self.__file.tell();
        let true_offset = (reqs_offset + req_offset);
        self.__file.seek(true_offset);
        let magic = self.get_int(true);
        if magic != mdictionary::signatures["REQUIREMENT"] {
            self.add_abnormality(
                "Bad magic \"{}\" for requirement at offset \"{}\"."
                    .format(magic, (self.__file.tell() - 4)),
            );
            return;
        }
        self.__file.read(8);
        self.__macho["code_signature"]["requirements"].append(
            [
                ("req_type", req_type),
                ("req_offset", req_offset),
                ("expression", self.parse_expression()),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        );
        self.__file.seek(prev);
    }
    fn parse_requirements<T0, T1>(&self, sig_offset: T0, index_offset: T1) {
        "Parse requirements from code signature.";
        let prev = self.__file.tell();
        let true_offset = (sig_offset + index_offset);
        self.__file.seek(true_offset);
        let magic = self.get_int(true);
        if magic != mdictionary::signatures["REQUIREMENTS"] {
            self.add_abnormality(
                "Bad magic \"{}\" for requirements at offset \"{}\"."
                    .format(magic, (self.__file.tell() - 4)),
            );
            return;
        }
        self.__file.read(4);
        let count = self.get_int(true);
        self.__macho["code_signature"]["requirements"] = vec![];
        for _ in (0..count) {
            let mut req_type = self.get_int(true);
            req_type = mdictionary::requirements[req_type];
            let req_offset = self.get_int(true);
            self.parse_requirement(true_offset, req_type, req_offset);
        }
        self.__file.seek(prev);
    }
    fn parse_sig<T0, T1, T2>(&self, offset: T0, size: T1, lc_codesig: T2) {
        "Parse code signature in its entirety.";
        if (lc_codesig["dataoff"] + lc_codesig["datasize"]) > size {
            self.add_abnormality(
                "CODE_SIGNATURE at offset \"{}\" with size \"{}\" greater than mach-o size."
                    .format((offset + lc_codesig["dataoff"]), lc_codesig["datasize"]),
            );
            return;
        }
        let true_offset = (offset + lc_codesig["dataoff"]);
        self.__file.seek(true_offset);
        let magic = self.get_int(true);
        if magic != mdictionary::signatures["EMBEDDED_SIGNATURE"] {
            self.add_abnormality(
                "Bad magic \"{}\" for embedded signature at offset \"{}\"."
                    .format(magic, true_offset),
            );
            return;
        }
        self.__macho["code_signature"] = HashMap::new();
        size = self.get_int(true);
        let count = self.get_int(true);
        for _ in (0..count) {
            let mut index_type = self.get_int(true);
            if mdictionary::indeces.iter().any(|&x| x == index_type) {
                index_type = mdictionary::indeces[index_type];
            } else {
                self.add_abnormality(
                    "Unknown code signature index type \"{}\" at offset \"{}\"."
                        .format(index_type, (self.__file.tell() - 4)),
                );
                self.__file.read(4);
                continue;
            }
            let index_offset = self.get_int(true);
            if index_type == "SignatureSlot" {
                self.parse_certs(true_offset, index_offset);
            } else {
                if index_type == "CodeDirectorySlot" {
                    self.parse_codedirectory(true_offset, index_offset);
                } else {
                    if index_type == "EntitlementSlot" {
                        self.parse_entitlement(true_offset, index_offset);
                    } else {
                        if index_type == "RequirementsSlot" {
                            self.parse_requirements(true_offset, index_offset);
                        }
                    }
                }
            }
        }
    }
    fn parse_macho<T0, T1, RT>(&self, offset: T0, size: T1) -> RT {
        "Parse mach-o binary, possibly contained within a
        universal binary.
        ";
        if size == None {
            self.__file.seek(0, SEEK_END);
            size = self.__file.tell();
        }
        self.__file.seek(offset);
        let identity = self.identify_file();
        self.__is_64_bit = identity[0];
        self.__is_little_endian = identity[1];
        let mut cputype = self.get_int();
        let mut subtype = self.get_int();
        let mut filetype = self.get_int();
        let nlcs = self.get_int();
        let slcs = self.get_int();
        let mut flags = self.get_int();
        if self.__is_64_bit {
            self.__file.read(4);
        }
        if mdictionary::cputypes.iter().any(|&x| x == cputype) {
            if mdictionary::cputypes[cputype].iter().any(|&x| x == subtype) {
                subtype = mdictionary::cputypes[cputype][subtype];
            } else {
                self.add_abnormality(
                    "Unknown SUBTYPE \"{}\" for CPUTYPE \"{}\" at offset \"{}\".".format(
                        subtype,
                        cputype,
                        (offset + 8),
                    ),
                );
            }
            cputype = mdictionary::cputypes[cputype][-2];
        } else {
            raise!(ValueError(
                "Unknown or unsupported CPUTYPE \"{}\" at offset \"{}\"."
                    .format(cputype, (offset + 4))
            )); //unsupported
        }
        if mdictionary::filetypes.iter().any(|&x| x == filetype) {
            filetype = mdictionary::filetypes[filetype];
        } else {
            self.add_abnormality(
                "Unknown FILETYPE \"{}\" at offset \"{}\".".format(filetype, (offset + 12)),
            );
        }
        let mut flags = self.parse_macho_flags(flags);
        self.__macho["cputype"] = cputype;
        self.__macho["subtype"] = subtype;
        self.__macho["filetype"] = filetype;
        self.__macho["nlcs"] = nlcs;
        self.__macho["slcs"] = slcs;
        self.__macho["flags"] = flags;
        self.parse_lcs(offset, size, nlcs, slcs);
        let lcs = self.__macho["lcs"]
            .iter()
            .map(|x| x["cmd"])
            .collect::<Vec<_>>();
        if lcs.iter().any(|&x| x == "SYMTAB") {
            let lc_symtab = self.__macho["lcs"][lcs.index("SYMTAB")];
            self.parse_syms(offset, size, lc_symtab);
            self.parse_strings(offset, size, lc_symtab);
        }
        if self.__macho.iter().any(|&x| x == "symtab")
            && self.__macho.iter().any(|&x| x == "strtab")
        {
            let mut lc_dysymtab = None;
            let mut lc_dylibs = None;
            if lcs.iter().any(|&x| x == "DYSYMTAB") {
                lc_dysymtab = self.__macho["lcs"][lcs.index("DYSYMTAB")];
                self.dysymtab = lc_dysymtab;
            }
            if self.__macho["flags"].iter().any(|&x| x == "TWOLEVEL") {
                lc_dylibs = self.__macho["lcs"]
                    .into_iter()
                    .filter(|x| x["cmd"].endswith("DYLIB"))
                    .collect::<Vec<_>>();
            }
            self.parse_imports(offset, size, lc_symtab, lc_dysymtab, lc_dylibs);
        }
        if lcs.iter().any(|&x| x == "CODE_SIGNATURE") {
            let lc_codesig = self.__macho["lcs"][lcs.index("CODE_SIGNATURE")];
        }
        self.__macho["imports"] = None;
        return self.__macho;
    }
    fn parse_universal(&self) {
        "Parses universal binary.";
        self.__output["universal"] = [("machos", vec![])]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>();
        let n_machos = self.get_int(true);
        for i in (0..n_machos) {
            self.__file.read(8);
            let offset = self.get_int(true);
            let size = self.get_int(true);
            self.__file.read(4);
            let prev = self.__file.tell();
            self.parse_macho(offset, size);
            self.__file.seek(prev);
            self.__output["universal"]["machos"].append(self.__macho.copy());
            self.__macho.clear();
        }
    }
    fn u_get_offset<T0, T1, RT>(&self, cpu_type: T0, uni_index: T1) -> RT {
        self.__file.seek(0);
        if self.__file.read(4) != b"\xca\xfe\xba\xbe" {
            logger.critical("Wrong magic for universal binary?");
        }
        let n_machos = self.get_int(true);
        for i in (0..n_machos) {
            self.__file.read(8);
            let offset = self.get_int(true);
            let size = self.get_int(true);
            self.__file.read(4);
            let old = self.__file.tell();
            self.__file.seek(offset);
            let identity = self.identify_file();
            self.__is_64_bit = identity[0];
            self.__is_little_endian = identity[1];
            let mut cputype = self.get_int();
            let mut subtype = self.get_int();
            if mdictionary::cputypes.iter().any(|&x| x == cputype) {
                if mdictionary::cputypes[cputype].iter().any(|&x| x == subtype) {
                    subtype = mdictionary::cputypes[cputype][subtype];
                }
            } else {
                logger.debug(("UNKNOWN CPU TYPE: " + String::from(cputype)));
            }
            cputype = mdictionary::cputypes[cputype][-2];
            self.__file.seek(old);
            if i == uni_index || cpu_type == cputype {
                return (offset, size);
            }
        }
    }
    fn parse_file(&self) {
        "Determines characteristics about the entire file and begins
        to parse.
        ";
        let contents = self.__file.read();
        self.__output["size"] = contents.len();
        self.__output["hashes"] = [
            ("md5", hashlib.md5(contents).hexdigest()),
            ("sha1", hashlib.sha1(contents).hexdigest()),
            ("sha256", hashlib.sha256(contents).hexdigest()),
        ]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
        self.__file.seek(0);
        if self.__file.read(4) == b"\xca\xfe\xba\xbe" {
            self.parse_universal();
        } else {
            self.parse_macho(0, self.__output["size"]);
            self.__output["macho"] = self.__macho;
        }
    }
    fn parse<T0, RT>(&self, certs: bool, out: T0) -> RT {
        "Parse Mach-O file at given path, and either return a dict
        or write output to provided file.
        ";
        self.__extract_certs = certs;
        self.parse_file();
        if out == None {
            return self.__output;
        }
        dump(self.__output, out);
    }
}
struct mdictionary {}

impl mdictionary {
    const cert_slots: _ = [(-1, "root"), (0, "leaf")]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
    const hashes: _ = [(0, "No Hash"), (1, "SHA-1"), (2, "SHA-256")]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
    const segment_flags: _ = [
        (1, "HIGHVM"),
        (2, "FVMLIB"),
        (4, "NORELOC"),
        (8, "PROTECTED_VERSION_1"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const n_types: _ = [
        (0, "UNDF"),
        (2, "ABS"),
        (14, "SECT"),
        (12, "PBUD"),
        (10, "INDR"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const machos: _ = [
        (4277009102, (false, false)),
        (4277009103, (true, false)),
        (3472551422, (false, true)),
        (3489328638, (true, true)),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const requirements: _ = [
        (1, "HostRequirementType"),
        (2, "GuestRequirementType"),
        (3, "DesignatedRequirementType"),
        (4, "LibraryRequirementType"),
        (5, "PluginRequirementType"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const indeces: _ = [
        (0, "CodeDirectorySlot"),
        (1, "InfoSlot"),
        (2, "RequirementsSlot"),
        (3, "ResourceDirSlot"),
        (4, "ApplicationSlot"),
        (5, "EntitlementSlot"),
        (65536, "SignatureSlot"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const matches: _ = [
        (0, "matchExists"),
        (1, "matchEqual"),
        (2, "matchContains"),
        (3, "matchBeginsWith"),
        (4, "matchEndsWith"),
        (5, "matchLessThan"),
        (6, "matchGreaterThan"),
        (7, "matchLessEqual"),
        (8, "matchGreaterEqual"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const protections: _ = [
        (0, "---"),
        (1, "r--"),
        (2, "-w-"),
        (3, "rw-"),
        (4, "--x"),
        (5, "r-x"),
        (6, "-wx"),
        (7, "rwx"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const signatures: _ = [
        ("REQUIREMENT", 4208856064),
        ("REQUIREMENTS", 4208856065),
        ("CODEDIRECTORY", 4208856066),
        ("ENTITLEMENT", 4208882033),
        ("BLOBWRAPPER", 4208855809),
        ("EMBEDDED_SIGNATURE", 4208856256),
        ("DETACHED_SIGNATURE", 4208856257),
        ("CODE_SIGN_DRS", 4208856069),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const section_attrs: _ = [
        (2147483648, "PURE_INSTRUCTIONS"),
        (1073741824, "NO_TOC"),
        (536870912, "STRIP_STATIC_SYMS"),
        (268435456, "NO_DEAD_STRIP"),
        (134217728, "LIVE_SUPPORT"),
        (67108864, "SELF_MODIFYING_CODE"),
        (33554432, "DEBUG"),
        (1024, "SOME_INSTRUCTIONS"),
        (512, "EXT_RELOC"),
        (256, "LOC_RELOC"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const filetypes: _ = [
        (1, "OBJECT"),
        (2, "EXECUTE"),
        (3, "FVMLIB"),
        (4, "CORE"),
        (5, "PRELOAD"),
        (6, "DYLIB"),
        (7, "DYLINKER"),
        (8, "BUNDLE"),
        (9, "DYLIB_STUB"),
        (10, "DSYM"),
        (11, "KEXT_BUNDLE"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const section_types: _ = [
        (0, "REGULAR"),
        (1, "ZEROFILL"),
        (2, "CSTRING_LITERALS"),
        (3, "4BYTE_LITERALS"),
        (4, "8BYTE_LITERALS"),
        (5, "LITERAL_POINTERS"),
        (6, "NON_LAZY_SYMBOL_POINTERS"),
        (7, "LAZY_SYMBOL_POINTERS"),
        (8, "SYMBOL_STUBS"),
        (9, "MOD_INIT_FUNC_POINTERS"),
        (10, "MOD_TERM_FUNC_POINTERS"),
        (11, "COALESCED"),
        (12, "GB_ZEROFILL"),
        (13, "INTERPOSING"),
        (14, "16BYTE_LITERALS"),
        (15, "DTRACE_DOF"),
        (16, "LAZY_DYLIB_SYMBOL_POINTERS"),
        (17, "THREAD_LOCAL_REGULAR"),
        (18, "THREAD_LOCAL_ZEROFILL"),
        (19, "THREAD_LOCAL_VARIABLES"),
        (20, "THREAD_LOCAL_VARIABLE_POINTERS"),
        (21, "THREAD_LOCAL_INIT_FUNCTION_POINTERS"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const operators: _ = [
        (0, "False"),
        (1, "True"),
        (2, "Ident"),
        (3, "AppleAnchor"),
        (4, "AnchorHash"),
        (5, "InfoKeyValue"),
        (6, "And"),
        (7, "Or"),
        (8, "CDHash"),
        (9, "Not"),
        (10, "InfoKeyField"),
        (11, "CertField"),
        (12, "TrustedCert"),
        (13, "TrustedCerts"),
        (14, "CertGeneric"),
        (15, "AppleGenericAnchor"),
        (16, "EntitlementField"),
        (17, "CertPolicy"),
        (18, "NamedAnchor"),
        (19, "NamedCode"),
        (20, "Platform"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const thread_states: _ = [
        (1, "x86_THREAD_STATE32"),
        (2, "x86_FLOAT_STATE32"),
        (3, "x86_EXCEPTION_STATE32"),
        (4, "x86_THREAD_STATE64"),
        (5, "x86_FLOAT_STATE64"),
        (6, "x86_EXCEPTION_STATE64"),
        (7, "x86_THREAD_STATE"),
        (8, "x86_FLOAT_STATE"),
        (9, "x86_EXCEPTION_STATE"),
        (10, "x86_DEBUG_STATE32"),
        (11, "x86_DEBUG_STATE64"),
        (12, "x86_DEBUG_STATE"),
        (13, "THREAD_STATE_NONE"),
        (14, "x86_SAVED_STATE_1 (INTERNAL ONLY)"),
        (15, "x86_SAVED_STATE_2 (INTERNAL ONLY)"),
        (16, "x86_AVX_STATE32"),
        (17, "x86_AVX_STATE64"),
        (18, "x86_AVX_STATE"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const flags: _ = [
        (1, "NOUNDEFS"),
        (2, "INCRLINK"),
        (4, "DYLDLINK"),
        (8, "BINDATLOAD"),
        (16, "PREBOUND"),
        (32, "SPLIT_SEGS"),
        (64, "LAZY_INIT"),
        (128, "TWOLEVEL"),
        (256, "FORCE_FLAT"),
        (512, "NOMULTIDEFS"),
        (1024, "NOFIXPREBINDING"),
        (2048, "PREBINDABLE"),
        (4096, "ALLMODSBOUND"),
        (8192, "SUBSECTIONS_VIA_SYMBOLS"),
        (16384, "CANONICAL"),
        (32768, "WEAK_DEFINES"),
        (65536, "BINDS_TO_WEAK"),
        (131072, "ALLOW_STACK_EXECUTION"),
        (262144, "ROOT_SAFE"),
        (524288, "SETUID_SAFE"),
        (1048576, "NOREEXPORTED_DYLIBS"),
        (2097152, "PIE"),
        (4194304, "DEAD_STRIPPABLE_DYLIB"),
        (8388608, "HAS_TLV_DESCRIPTORS"),
        (16777216, "NO_HEAP_EXECUTION"),
        (33554432, "APP_EXTENSION_SAFE"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const stabs: _ = [
        (32, "GSYM"),
        (34, "FNAME"),
        (36, "FUN"),
        (38, "STSYM"),
        (40, "LCSYM"),
        (42, "MAIN"),
        (46, "BNSYM"),
        (48, "PC"),
        (50, "AST"),
        (58, "MAC_UNDEF"),
        (60, "OPT"),
        (64, "RSYM"),
        (68, "SLINE"),
        (70, "DSLINE"),
        (72, "BSLINE"),
        (78, "ENSYM"),
        (96, "SSYM"),
        (100, "SO"),
        (102, "OSO"),
        (128, "LSYM"),
        (130, "BINCL"),
        (132, "SOL"),
        (134, "PARAMS"),
        (136, "VERSION"),
        (138, "OLEVEL"),
        (160, "PSYM"),
        (162, "EINCL"),
        (164, "ENTRY"),
        (192, "LBRAC"),
        (194, "EXCL"),
        (224, "RBRAC"),
        (226, "BCOMM"),
        (228, "ECOMM"),
        (232, "ECOML"),
        (254, "LENG"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const loadcommands: _ = [
        (1, "SEGMENT"),
        (2, "SYMTAB"),
        (3, "SYMSEG"),
        (4, "THREAD"),
        (5, "UNIXTHREAD"),
        (6, "LOADFVMLIB"),
        (7, "IDFVMLIB"),
        (8, "IDENT"),
        (9, "FVMFILE"),
        (10, "PREPAGE"),
        (11, "DYSYMTAB"),
        (12, "LOAD_DYLIB"),
        (13, "ID_DYLIB"),
        (14, "LOAD_DYLINKER"),
        (15, "ID_DYLINKER"),
        (16, "PREBOUND_DYLIB"),
        (17, "ROUTINES"),
        (18, "SUB_FRAMEWORK"),
        (19, "SUB_UMBRELLA"),
        (20, "SUB_CLIENT"),
        (21, "SUB_LIBRARY"),
        (22, "TWOLEVEL_HINTS"),
        (23, "PREBIND_CKSUM"),
        (25, "SEGMENT_64"),
        (26, "ROUTINES_64"),
        (27, "UUID"),
        (29, "CODE_SIGNATURE"),
        (30, "SEGMENT_SPLIT_INFO"),
        (32, "LAZY_LOAD_DYLIB"),
        (33, "ENCRYPTION_INFO"),
        (34, "DYLD_INFO"),
        (36, "VERSION_MIN_MACOSX"),
        (37, "VERSION_MIN_IPHONEOS"),
        (38, "FUNCTION_STARTS"),
        (39, "DYLD_ENVIRONMENT"),
        (41, "DATA_IN_CODE"),
        (42, "SOURCE_VERSION"),
        (43, "DYLIB_CODE_SIGN_DRS"),
        (44, "ENCRYPTION_INFO_64"),
        (45, "LINKER_OPTION"),
        (46, "LINKER_OPTIMIZATION_HINT"),
        (47, "VERSION_MIN_TVOS"),
        (48, "VERSION_MIN_WATCHOS"),
        (49, "NOTE"),
        (50, "BUILD_VERSION"),
        (2147483672, "LOAD_WEAK_DYLIB"),
        (2147483676, "RPATH"),
        (2147483679, "REEXPORT_DYLIB"),
        (2147483682, "DYLD_INFO_ONLY"),
        (2147483683, "LOAD_UPWARD_DYLIB"),
        (2147483688, "MAIN"),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    const cputypes: _ = [
        (
            -1,
            [
                (-2, "ANY"),
                (-1, "MULTIPLE"),
                (0, "LITTLE_ENDIAN"),
                (1, "BIG_ENDIAN"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            1,
            [
                (-2, "VAX"),
                (-1, "MULTIPLE"),
                (0, "VAX_ALL"),
                (1, "VAX780"),
                (2, "VAX785"),
                (3, "VAX750"),
                (4, "VAX730"),
                (5, "UVAXI"),
                (6, "UVAXII"),
                (7, "VAX8200"),
                (8, "VAX8500"),
                (9, "VAX8600"),
                (10, "VAX8650"),
                (11, "VAX8800"),
                (12, "UVAXIII"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            6,
            [
                (-2, "MC680x0"),
                (-1, "MULTIPLE"),
                (1, "MC680x0_ALL or MC68030"),
                (2, "MC68040"),
                (3, "MC68030_ONLY"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            7,
            [
                (-2, "X86 (I386)"),
                (-1, "MULITPLE"),
                (0, "INTEL_MODEL_ALL"),
                (3, "X86_ALL, X86_64_ALL, I386_ALL, or 386"),
                (4, "X86_ARCH1 or 486"),
                (5, "586 or PENT"),
                (8, "X86_64_H or PENTIUM_3"),
                (9, "PENTIUM_M"),
                (10, "PENTIUM_4"),
                (11, "ITANIUM"),
                (12, "XEON"),
                (15, "INTEL_FAMILY_MAX"),
                (22, "PENTPRO"),
                (24, "PENTIUM_3_M"),
                (26, "PENTIUM_4_M"),
                (27, "ITANIUM_2"),
                (28, "XEON_MP"),
                (40, "PENTIUM_3_XEON"),
                (54, "PENTII_M3"),
                (86, "PENTII_M5"),
                (103, "CELERON"),
                (119, "CELERON_MOBILE"),
                (132, "486SX"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            10,
            [
                (-2, "MC98000"),
                (-1, "MULTIPLE"),
                (0, "MC98000_ALL"),
                (1, "MC98601"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            11,
            [
                (-2, "HPPA"),
                (-1, "MULITPLE"),
                (0, "HPPA_ALL or HPPA_7100"),
                (1, "HPPA_7100LC"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            12,
            [
                (-2, "ARM"),
                (-1, "MULTIPLE"),
                (0, "ARM_ALL"),
                (1, "ARM_A500_ARCH"),
                (2, "ARM_A500"),
                (3, "ARM_A440"),
                (4, "ARM_M4"),
                (5, "ARM_V4T"),
                (6, "ARM_V6"),
                (7, "ARM_V5TEJ"),
                (8, "ARM_XSCALE"),
                (9, "ARM_V7"),
                (10, "ARM_V7F"),
                (11, "ARM_V7S"),
                (12, "ARM_V7K"),
                (13, "ARM_V8"),
                (14, "ARM_V6M"),
                (15, "ARM_V7M"),
                (16, "ARM_V7EM"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            13,
            [
                (-2, "MC88000"),
                (-1, "MULTIPLE"),
                (0, "MC88000_ALL"),
                (1, "MMAX_JPC or MC88100"),
                (2, "MC88110"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            14,
            [
                (-2, "SPARC"),
                (-1, "MULTIPLE"),
                (0, "SPARC_ALL or SUN4_ALL"),
                (1, "SUN4_260"),
                (2, "SUN4_110"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            15,
            [
                (-2, "I860 (big-endian)"),
                (-1, "MULTIPLE"),
                (0, "I860_ALL"),
                (1, "I860_860"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            18,
            [
                (-2, "POWERPC"),
                (-1, "MULTIPLE"),
                (0, "POWERPC_ALL"),
                (1, "POWERPC_601"),
                (2, "POWERPC_602"),
                (3, "POWERPC_603"),
                (4, "POWERPC_603e"),
                (5, "POWERPC_603ev"),
                (6, "POWERPC_604"),
                (7, "POWERPC_604e"),
                (8, "POWERPC_620"),
                (9, "POWERPC_750"),
                (10, "POWERPC_7400"),
                (11, "POWERPC_7450"),
                (100, "POWERPC_970"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            16777223,
            [
                (-2, "X86_64"),
                (-1, "MULTIPLE"),
                (0, "INTEL_MODEL_ALL"),
                (3, "X86_ALL, X86_64_ALL, I386_ALL, or 386"),
                (4, "X86_ARCH1 or 486"),
                (5, "586 or PENT"),
                (8, "X86_64_H or PENTIUM_3"),
                (9, "PENTIUM_M"),
                (10, "PENTIUM_4"),
                (11, "ITANIUM"),
                (12, "XEON"),
                (15, "INTEL_FAMILY_MAX"),
                (22, "PENTPRO"),
                (24, "PENTIUM_3_M"),
                (26, "PENTIUM_4_M"),
                (27, "ITANIUM_2"),
                (28, "XEON_MP"),
                (40, "PENTIUM_3_XEON"),
                (54, "PENTII_M3"),
                (86, "PENTII_M5"),
                (103, "CELERON"),
                (119, "CELERON_MOBILE"),
                (132, "486SX"),
                ((2147483648 + 0), "INTEL_MODEL_ALL"),
                ((2147483648 + 3), "X86_ALL, X86_64_ALL, I386_ALL, or 386"),
                ((2147483648 + 4), "X86_ARCH1 or 486"),
                ((2147483648 + 5), "586 or PENT"),
                ((2147483648 + 8), "X86_64_H or PENTIUM_3"),
                ((2147483648 + 9), "PENTIUM_M"),
                ((2147483648 + 10), "PENTIUM_4"),
                ((2147483648 + 11), "ITANIUM"),
                ((2147483648 + 12), "XEON"),
                ((2147483648 + 15), "INTEL_FAMILY_MAX"),
                ((2147483648 + 22), "PENTPRO"),
                ((2147483648 + 24), "PENTIUM_3_M"),
                ((2147483648 + 26), "PENTIUM_4_M"),
                ((2147483648 + 27), "ITANIUM_2"),
                ((2147483648 + 28), "XEON_MP"),
                ((2147483648 + 40), "PENTIUM_3_XEON"),
                ((2147483648 + 54), "PENTII_M3"),
                ((2147483648 + 86), "PENTII_M5"),
                ((2147483648 + 103), "CELERON"),
                ((2147483648 + 119), "CELERON_MOBILE"),
                ((2147483648 + 132), "486SX"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            16777228,
            [
                (-2, "ARM64"),
                (-1, "MULTIPLE"),
                (0, "ARM64_ALL"),
                (1, "ARM64_V8"),
                ((2147483648 + 0), "ARM64_ALL"),
                ((2147483648 + 1), "ARM64_V8"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
        (
            16777234,
            [
                (-2, "POWERPC64"),
                (-1, "MULTIPLE"),
                (0, "POWERPC_ALL"),
                (1, "POWERPC_601"),
                (2, "POWERPC_602"),
                (3, "POWERPC_603"),
                (4, "POWERPC_603e"),
                (5, "POWERPC_603ev"),
                (6, "POWERPC_604"),
                (7, "POWERPC_604e"),
                (8, "POWERPC_620"),
                (9, "POWERPC_750"),
                (10, "POWERPC_7400"),
                (11, "POWERPC_7450"),
                (100, "POWERPC_970"),
                ((2147483648 + 0), "POWERPC_ALL (LIB64)"),
                ((2147483648 + 1), "POWERPC_601 (LIB64)"),
                ((2147483648 + 2), "POWERPC_602 (LIB64)"),
                ((2147483648 + 3), "POWERPC_603 (LIB64)"),
                ((2147483648 + 4), "POWERPC_603e (LIB64)"),
                ((2147483648 + 5), "POWERPC_603ev (LIB64)"),
                ((2147483648 + 6), "POWERPC_604 (LIB64)"),
                ((2147483648 + 7), "POWERPC_604e (LIB64)"),
                ((2147483648 + 8), "POWERPC_620 (LIB64)"),
                ((2147483648 + 9), "POWERPC_750 (LIB64)"),
                ((2147483648 + 10), "POWERPC_7400 (LIB64)"),
                ((2147483648 + 11), "POWERPC_7450 (LIB64)"),
                ((2147483648 + 100), "POWERPC_970 (LIB64)"),
            ]
            .iter()
            .cloned()
            .collect::<HashMap<_, _>>(),
        ),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
}
