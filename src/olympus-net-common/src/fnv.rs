const OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0100_0000_01b3;

// FNV-1a hash
#[allow(clippy::must_use_candidate)]
pub const fn fnv(str: &str) -> u64 {
	let mut hash = OFFSET_BASIS;

	let bytes = str.as_bytes();
	let mut idx = 0;
	while idx < bytes.len() {
		hash ^= bytes[idx] as u64;
		hash = hash.wrapping_mul(PRIME);
		idx += 1;
	}

	hash
}

#[cfg(test)]
mod tests {
	use crate::fnv::fnv;

	#[test]
	fn works() {
		assert_eq!(fnv("a"), 0xaf63_dc4c_8601_ec8c);
		assert_eq!(fnv("abc"), 0xe71f_a219_0541_574b);
		assert_eq!(fnv("12345678"), 0x1739_32c4_1a90_a42d);
		assert_eq!(fnv("w54s6edr75tf8yg9uh0ij!@#^&*"), 0x2be8_d04f_8a4f_a8d2);
		assert_eq!(fnv("w3sd5e6rcfvt7yg8u54s123r6ftyguhij565t789yghui5867912367890asdhuio6edr75tf8yg9uh5436rtuyg&!)@#Y*HUPEINJW:0ij6@^#&!)(YE*UPHOQWD!@#^&*"), 0x9b93_71d5_ab6f_b467);
	}
}
