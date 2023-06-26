//! A xor-shift Random number generator

pub struct Random(u64);

impl Random {

    const PRINTABLE: &'static [u8] =
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstyvwxyz1234567890~!@#$%^&*()_+[];'./,{}:<>?`-=".as_bytes();

    pub fn rand(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 43;
        self.0
    }

    pub fn new(seed: u64) -> Self {
        let seed = if seed == 0 {
            let r = unsafe { std::arch::x86_64::_rdtsc() };
            // // println!("random value = {:x}",r);
            r
            // 0x88880009999
            // 0x8888
            // 0x64a967374dfaa098
        } else {
            seed
        };

        Self(seed)
    }

    pub fn _rand8(&mut self) -> u8 {
        self.rand() as u8
    }

    pub fn _rand32(&mut self) -> u32 {
        self.rand() as u32
    }

    pub fn _rand64(&mut self) -> u64 {
        self.rand() as u64
    }

    // Given a Vec or an array, return the reference to a random element in that
    // collection
    pub fn random_element<'a, U, T>(&mut self, array: &'a T) -> &'a U
        where T: AsRef<[U]> {

        let len = array.as_ref().len();
        let idx = self.rand_in_range(0, len as isize) as usize;
        &array.as_ref()[idx]
    }

    // Given a Vec or an array, return the a vector of references to n random
    // elements in that collection
    pub fn get_n_random_elements<'a, U, T>(&mut self, array: &'a T, n: usize)
                                           -> Vec<&'a U>
        where T: AsRef<[U]> {

        let array = array.as_ref();
        let len = array.len();

        let n = std::cmp::min(len, n);
        let mut out = Vec::<&'a U>::with_capacity(n);

        let mut temp = vec![false; len];

        while out.len() != n {
            let idx = self.rand_idx(len);
            if temp[idx] {continue}

            temp[idx] = true;
            out.push(&array[idx]);
        }

        out
    }

    pub fn rand_idx(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }

        self.rand() as usize % len
    }

    /// Returns a random number in the range of [min, max)
    pub fn rand_in_range(&mut self, min: isize, max: isize) -> isize {
        if min == max {
            return min;
        }

        min.wrapping_add((self.rand() % (max.wrapping_sub(min) as u64)) as isize)
    }

    pub fn float_in_range(&mut self, _min: isize, max: isize) -> f64 {
        let r = self.rand_in_range(i32::MIN as isize, i32::MAX as isize);
        let d = i32::MAX as f64 / max as f64;
        r as f64 / d
    }

    pub fn random_string(&mut self, len: u64) -> String {
        let length = Random::PRINTABLE.len();
        let mut s: String = String::new();
        for _ in 0..len {
            let i = self.rand_in_range(0, length as isize);
            s.push(Random::PRINTABLE[i as usize] as char);
        }

        s
    }

}
