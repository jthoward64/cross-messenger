use std::collections::HashMap;
use std::*;

const BINARY_HASH: _ = "e1181ccad82e6629d52c6a006645ad87ee59bd13";
const BINARY_PATH: _ = "emulated/IMDAppleServices";
const BINARY_URL: _ = "https://github.com/JJTech0130/nacserver/raw/main/IMDAppleServices";
const FAKE_DATA: _ = plistlib.load(open("emulated/data.plist", "rb"));
fn load_binary() -> &[u8] {
    if !os.path.exists(BINARY_PATH) {
        let resp = requests.get(BINARY_URL);
        b = resp.content;
        open(BINARY_PATH, "wb").write(b);
    } else {
        b = open(BINARY_PATH, "rb").read();
    }
    if hashlib.sha1(b).hexdigest() != BINARY_HASH {
        raise!(Exception("Hashes don't match")); //unsupported
    }
    return b;
}
// fn get_x64_slice(binary: &[u8]) -> &[u8] {
// let p = macholibre.Parser(binary);
// let (off, size) = p.u_get_offset("X86_64");
// return binary[off..(off + size)];
// }
fn nac_init<RT>(j: Jelly, cert: &[u8]) -> RT {
    let cert_addr = j.malloc(cert.len());
    j.uc.mem_write(cert_addr, cert);
    let out_validation_ctx_addr = j.malloc(8);
    let out_request_bytes_addr = j.malloc(8);
    let out_request_len_addr = j.malloc(8);
    let ret = j.instr.call(
        728496,
        vec![
            cert_addr,
            cert.len(),
            out_validation_ctx_addr,
            out_request_bytes_addr,
            out_request_len_addr,
        ],
    );
    if ret != 0 {
        let mut n = (ret & 4294967295);
        n = ((n ^ 2147483648) - 2147483648);
        raise!(Exception()); //unsupported
    }
    let mut validation_ctx_addr = j.uc.mem_read(out_validation_ctx_addr, 8);
    let mut request_bytes_addr = j.uc.mem_read(out_request_bytes_addr, 8);
    let mut request_len = j.uc.mem_read(out_request_len_addr, 8);
    request_bytes_addr = int.from_bytes(request_bytes_addr, "little");
    request_len = int.from_bytes(request_len, "little");
    let request = j.uc.mem_read(request_bytes_addr, request_len);
    validation_ctx_addr = int.from_bytes(validation_ctx_addr, "little");
    return (validation_ctx_addr, request);
}
fn nac_key_establishment(j: Jelly, validation_ctx: i32, response: &[u8]) {
    let response_addr = j.malloc(response.len());
    j.uc.mem_write(response_addr, response);
    let ret = j
        .instr
        .call(728528, vec![validation_ctx, response_addr, response.len()]);
    if ret != 0 {
        let mut n = (ret & 4294967295);
        n = ((n ^ 2147483648) - 2147483648);
        raise!(Exception()); //unsupported
    }
}
fn nac_sign<RT>(j: Jelly, validation_ctx: i32) -> RT {
    let out_validation_data_addr = j.malloc(8);
    let out_validation_data_len_addr = j.malloc(8);
    let ret = j.instr.call(
        728560,
        vec![
            validation_ctx,
            0,
            0,
            out_validation_data_addr,
            out_validation_data_len_addr,
        ],
    );
    if ret != 0 {
        let mut n = (ret & 4294967295);
        n = ((n ^ 2147483648) - 2147483648);
        raise!(Exception()); //unsupported
    }
    let mut validation_data_addr = j.uc.mem_read(out_validation_data_addr, 8);
    let mut validation_data_len = j.uc.mem_read(out_validation_data_len_addr, 8);
    validation_data_addr = int.from_bytes(validation_data_addr, "little");
    validation_data_len = int.from_bytes(validation_data_len, "little");
    let validation_data = j.uc.mem_read(validation_data_addr, validation_data_len);
    return validation_data;
}
fn malloc(j: Jelly, len: i32) -> i32 {
    return j.malloc(len);
}
fn memset_chk<RT>(j: Jelly, dest: i32, c: i32, len: i32, destlen: i32) -> RT {
    j.uc.mem_write(dest, (bytes(vec![c]) * len));
    return 0;
}
fn sysctlbyname<RT>(j: Jelly) -> RT {
    return 0;
}
fn memcpy<RT>(j: Jelly, dest: i32, src: i32, len: i32) -> RT {
    let orig = j.uc.mem_read(src, len);
    j.uc.mem_write(dest, bytes(orig));
    return 0;
}
// let CF_OBJECTS = vec![];
fn _parse_cfstr_ptr(j: Jelly, ptr: i32) -> &str {
    let size = struct_.calcsize("<QQQQ");
    let data = j.uc.mem_read(ptr, size);
    let (isa, flags, str_ptr, length) = struct_.unpack("<QQQQ", data);
    let str_data = j.uc.mem_read(str_ptr, length);
    return str_data.decode("utf-8");
}
fn _parse_cstr_ptr(j: Jelly, ptr: i32) -> &str {
    let data = j.uc.mem_read(ptr, 256);
    return data.split(b"\x00")[0].decode("utf-8");
}
fn IORegistryEntryCreateCFProperty<RT>(
    j: Jelly,
    entry: i32,
    key: i32,
    allocator: i32,
    options: i32,
) -> RT {
    let key_str = _parse_cfstr_ptr(j, key);
    if FAKE_DATA["iokit"].iter().any(|&x| x == key_str) {
        let fake = FAKE_DATA["iokit"][key_str];
        CF_OBJECTS.push(fake);
        return CF_OBJECTS.len();
    } else {
        return 0;
    }
}
fn CFGetTypeID<RT>(j: Jelly, obj: i32) -> RT {
    obj = CF_OBJECTS[(obj - 1)];
    if isinstance(obj, bytes) {
        return 1;
    } else {
        if isinstance(obj, str) {
            return 2;
        } else {
            raise!(Exception("Unknown CF object type")); //unsupported
        }
    }
}
fn CFDataGetLength<RT>(j: Jelly, obj: i32) -> RT {
    obj = CF_OBJECTS[(obj - 1)];
    if isinstance(obj, bytes) {
        return obj.len();
    } else {
        raise!(Exception("Unknown CF object type")); //unsupported
    }
}
fn CFDataGetBytes<RT>(j: Jelly, obj: i32, range_start: i32, range_end: i32, buf: i32) -> RT {
    obj = CF_OBJECTS[(obj - 1)];
    if isinstance(obj, bytes) {
        let data = obj[range_start..range_end];
        j.uc.mem_write(buf, data);
        return data.len();
    } else {
        raise!(Exception("Unknown CF object type")); //unsupported
    }
}
fn CFDictionaryCreateMutable(j: Jelly) -> i32 {
    CF_OBJECTS.push(HashMap::new());
    return CF_OBJECTS.len();
}
fn maybe_object_maybe_string<RT>(j: Jelly, obj: i32) -> RT {
    if isinstance(obj, str) {
        return obj;
    } else {
        if obj > CF_OBJECTS.len() {
            return obj;
        } else {
            return CF_OBJECTS[(obj - 1)];
        }
    }
}
fn CFDictionaryGetValue(j: Jelly, d: i32, key: i32) -> i32 {
    d = CF_OBJECTS[(d - 1)];
    if key == 14106333703424951235 {
        key = "DADiskDescriptionVolumeUUIDKey";
    }
    key = maybe_object_maybe_string(j, key);
    if isinstance(d, dict) {
        if d.iter().any(|&x| x == key) {
            let val = d[key];
            CF_OBJECTS.push(val);
            return CF_OBJECTS.len();
        } else {
            raise!(Exception("Key not found")); //unsupported
            return 0;
        }
    } else {
        raise!(Exception("Unknown CF object type")); //unsupported
    }
}
fn CFDictionarySetValue(j: Jelly, d: i32, key: i32, val: i32) {
    d = CF_OBJECTS[(d - 1)];
    key = maybe_object_maybe_string(j, key);
    val = maybe_object_maybe_string(j, val);
    if isinstance(d, dict) {
        d[key] = val;
    } else {
        raise!(Exception("Unknown CF object type")); //unsupported
    }
}
fn DADiskCopyDescription(j: Jelly) -> i32 {
    let description = CFDictionaryCreateMutable(j);
    CFDictionarySetValue(
        j,
        description,
        "DADiskDescriptionVolumeUUIDKey",
        FAKE_DATA["root_disk_uuid"],
    );
    return description;
}
fn CFStringCreate(j: Jelly, string: &str) -> i32 {
    CF_OBJECTS.push(string);
    return CF_OBJECTS.len();
}
fn CFStringGetLength(j: Jelly, string: i32) -> i32 {
    string = CF_OBJECTS[(string - 1)];
    if isinstance(string, str) {
        return string.len();
    } else {
        raise!(Exception("Unknown CF object type")); //unsupported
    }
}
fn CFStringGetCString(j: Jelly, string: i32, buf: i32, buf_len: i32, encoding: i32) -> i32 {
    string = CF_OBJECTS[(string - 1)];
    if isinstance(string, str) {
        let data = string.encode("utf-8");
        j.uc.mem_write(buf, data);
        return data.len();
    } else {
        raise!(Exception("Unknown CF object type")); //unsupported
    }
}
fn IOServiceMatching(j: Jelly, name: i32) -> i32 {
    name = _parse_cstr_ptr(j, name);
    name = CFStringCreate(j, name);
    let d = CFDictionaryCreateMutable(j);
    CFDictionarySetValue(j, d, "IOProviderClass", name);
    return d;
}
fn IOServiceGetMatchingService(j: Jelly) -> i32 {
    return 92;
}
const ETH_ITERATOR_HACK: _ = false;
// fn IOServiceGetMatchingServices<T0, T1, T2>(j: Jelly, port: T0, match: T1, existing: T2) -> i32 {
// //global ETH_ITERATOR_HACK
// ETH_ITERATOR_HACK = true;
// j.uc.mem_write(existing, bytes(vec![93]));
// return 0;
// }
fn IOIteratorNext(j: Jelly, iterator: i32) -> i32 {
    //global ETH_ITERATOR_HACK
    if ETH_ITERATOR_HACK {
        ETH_ITERATOR_HACK = false;
        return 94;
    } else {
        return 0;
    }
}
fn bzero<RT>(j: Jelly, ptr: i32, len: i32) -> RT {
    j.uc.mem_write(ptr, (bytes(vec![0]) * len));
    return 0;
}
fn IORegistryEntryGetParentEntry<T0>(j: Jelly, entry: i32, _: T0, parent: i32) -> i32 {
    j.uc.mem_write(parent, bytes(vec![(entry + 100)]));
    return 0;
}
fn get_cert<RT>() -> RT {
    let mut resp = requests.get("http://static.ess.apple.com/identity/validation/cert-1.0.plist");
    resp = plistlib.loads(resp.content);
    return resp["cert"];
}
fn get_session_info(req: &[u8]) -> &[u8] {
    let mut body = [("session-info-request", req)]
        .iter()
        .cloned()
        .collect::<HashMap<_, _>>();
    body = plistlib.dumps(body);
    let mut resp = requests.post(
        "https://identity.ess.apple.com/WebObjects/TDIdentityService.woa/wa/initializeValidation",
        body,
        false,
    );
    resp = plistlib.loads(resp.content);
    return resp["session-info"];
}
fn arc4random(j: Jelly) -> i32 {
    return random.randint(0, 4294967295);
}
fn load_nac() -> Jelly {
    let mut binary = load_binary();
    binary = get_x64_slice(binary);
    let j = Jelly(binary);
    let hooks = [
        ("_malloc", malloc),
        ("___stack_chk_guard", || 0),
        ("___memset_chk", memset_chk),
        ("_sysctlbyname", |_| 0),
        ("_memcpy", memcpy),
        ("_kIOMasterPortDefault", || 0),
        ("_IORegistryEntryFromPath", |_| 1),
        ("_kCFAllocatorDefault", || 0),
        (
            "_IORegistryEntryCreateCFProperty",
            IORegistryEntryCreateCFProperty,
        ),
        ("_CFGetTypeID", CFGetTypeID),
        ("_CFStringGetTypeID", |_| 2),
        ("_CFDataGetTypeID", |_| 1),
        ("_CFDataGetLength", CFDataGetLength),
        ("_CFDataGetBytes", CFDataGetBytes),
        ("_CFRelease", |_| 0),
        ("_IOObjectRelease", |_| 0),
        ("_statfs$INODE64", |_| 0),
        ("_DASessionCreate", |_| 201),
        ("_DADiskCreateFromBSDName", |_| 202),
        ("_kDADiskDescriptionVolumeUUIDKey", || 0),
        ("_DADiskCopyDescription", DADiskCopyDescription),
        ("_CFDictionaryGetValue", CFDictionaryGetValue),
        ("_CFUUIDCreateString", |_, __, uuid| uuid),
        ("_CFStringGetLength", CFStringGetLength),
        ("_CFStringGetMaximumSizeForEncoding", |_, length, __| length),
        ("_CFStringGetCString", CFStringGetCString),
        ("_free", |_| 0),
        ("_IOServiceMatching", IOServiceMatching),
        ("_IOServiceGetMatchingService", IOServiceGetMatchingService),
        ("_CFDictionaryCreateMutable", CFDictionaryCreateMutable),
        ("_kCFBooleanTrue", || 0),
        ("_CFDictionarySetValue", CFDictionarySetValue),
        (
            "_IOServiceGetMatchingServices",
            IOServiceGetMatchingServices,
        ),
        ("_IOIteratorNext", IOIteratorNext),
        ("___bzero", bzero),
        (
            "_IORegistryEntryGetParentEntry",
            IORegistryEntryGetParentEntry,
        ),
        ("_arc4random", arc4random),
    ]
    .iter()
    .cloned()
    .collect::<HashMap<_, _>>();
    j.setup(hooks);
    return j;
}
fn generate_validation_data() -> &[u8] {
    let j = load_nac();
    let (val_ctx, req) = nac_init(j, get_cert());
    let session_info = get_session_info(req);
    nac_key_establishment(j, val_ctx, session_info);
    let val_data = nac_sign(j, val_ctx);
    return bytes(val_data);
}
fn main() {
    use base64::b64encode;
    let val_data = generate_validation_data();
}
