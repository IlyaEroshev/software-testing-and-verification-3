namespace auxiliary_functions {
	uint64_t pow(uint64_t b, uint64_t e) {//b^e по модулю 2^64
		uint64_t res = (e & 1) ? b : 1;
		e >>= 1;
		while(e) {
			b *= b;
			if(e & 1) res *= b;
			e >>= 1;
		}
		return res;
	}

	uint64_t inv(uint64_t a) {// (1 / a) по модулю 2^64, а - нечетное
		return pow(a, 9223372036854775807ull);
	}

	void print_as_characters(std::ostream &out, uint64_t buf, int n = 8) {
		for(int i = 7; i >= 8-n; --i)
			out << static_cast<uint8_t>((buf >> (i * 8)) % 256);
	}
}

