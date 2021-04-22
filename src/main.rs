use std::io::{self, Write};

const TUPLE_LEN: usize = 10;
const MAX_PRIME: usize = 800;
const MAX_PRIME_OVER_2: usize = MAX_PRIME / 2;
const MIN_PRIME: usize = 3;

const SIEVE_SIZE: usize = 1000000;
const MAX_GAP: usize = 100000000;

// 3, 5 at 4
// 7 at 10
// 11 at 4, 14, 16
// So mod 2310 we have (1271, 1481, 1691)
const OFFSETS: [u32; TUPLE_LEN] = [0,  2,  6,  8,  12,  18,  20,  26,  30,  32];
const BASE: u32 = 1271;

struct PrimeAndOffset
{
	prime: u32,
	offset: [usize; TUPLE_LEN],
}

fn get_primes() -> [u8; MAX_PRIME_OVER_2]
{
    let mut sieve: [u8; MAX_PRIME_OVER_2] = [0; MAX_PRIME_OVER_2];
	
	for p in (MIN_PRIME..MAX_PRIME).step_by(2)
	{
		if sieve[p >> 1] == 0 
		{
			for q in ((p + (p >> 1))..MAX_PRIME_OVER_2).step_by(p)
			{
				sieve[q] = 1;
			}
		}
	}
	
	sieve
}

fn mod_inverse(n: u32, p: u32) -> u32
{
	// Stupid version is fine
	for i in 1..p-1
	{
		if (i * n) % p == 1
		{
			return i
		}
	}
	
	panic!("Failed modular inverse for {} % {}", n, p);
}

// Tuple is of form 2310f + o
// Offsets solve
// f_i = o.2310^-1 % p
fn make_offsets(prime_sieve: [u8; MAX_PRIME_OVER_2]) -> Vec<PrimeAndOffset>
{
	let mut poff: Vec<PrimeAndOffset> = Vec::new();
	
	for i in 6..MAX_PRIME_OVER_2
	{
		if prime_sieve[i] == 0
		{
			let p = (i*2 + 1) as u32;
			let prim_mod_p = 2310 % p;
			let prim_inv = mod_inverse(prim_mod_p, p);
			let mut offset: [usize; TUPLE_LEN] = [0; TUPLE_LEN];
			
			for i in 0..TUPLE_LEN {
				offset[i] = (((OFFSETS[i] + BASE) * prim_inv) % p) as usize;
				if offset[i] != 0 { 
					offset[i] = p as usize - offset[i]; 
				}
				assert!(((offset[i] * 2310) as u32 + OFFSETS[i] + BASE) % p == 0);
			}
			
			poff.push(PrimeAndOffset {
				prime: p,
				offset: offset,
			});
		}
	}
	
	poff
}

fn run_sieve(poff_arr: &mut Vec<PrimeAndOffset>) -> Vec<usize>
{
	let mut sieve: [u8; SIEVE_SIZE] = [0; SIEVE_SIZE];

	for poff in poff_arr.iter_mut()
	{
		let p = poff.prime as usize;
		for i in 0..TUPLE_LEN
		{
			for j in (poff.offset[i]..SIEVE_SIZE).step_by(p)
			{
				sieve[j] = 1;
			}
			let sieve_adjust = SIEVE_SIZE % p;
			if poff.offset[i] < sieve_adjust { 
				poff.offset[i] += p - sieve_adjust; 
			}
			else {
				poff.offset[i] -= sieve_adjust;
			}
		}
	}
	
	let mut results : Vec<usize> = Vec::new();
	for i in 0..SIEVE_SIZE
	{
		if sieve[i] == 0
		{
			results.push(i);
		}
	}
	
	results
}

fn tuple_from_offset(offset: usize) -> usize
{
	offset * 2310 + BASE as usize
}

fn main() 
{
	let prime_sieve = get_primes();
	let mut poff_arr = make_offsets(prime_sieve);
	
	let mut tuples: Vec<usize> = Vec::new();

	for batch in 0..200000
	{
		let results = run_sieve(&mut poff_arr);
		
		for i in results
		{
			tuples.push(i + batch * SIEVE_SIZE);
		}
		
		print!("{}\r", batch);
		io::stdout().flush().unwrap();
	}
	println!("Done sieving");
	
	use multimap::MultiMap;

	let mut gaps = MultiMap::new();
	for i in 0..tuples.len()
	{
		for j in i+1..tuples.len()
		{
			let gap = tuples[j] - tuples[i];
			if gap > MAX_GAP
			{
				break;
			}
			
			if tuples[i] > gap
			{
				if let Some(first_tuple_vec) = gaps.get_vec(&gap)
				{
					if first_tuple_vec.contains(&(tuples[i] - gap))
					{
						if tuples.contains(&(tuples[j] + gap))
						{
						println!("Gap: {} Tuples: {} {} {}", gap, tuple_from_offset(tuples[i] - gap), tuple_from_offset(tuples[i]), tuple_from_offset(tuples[j]));
							println!("*** Set of 4! {}", tuple_from_offset(tuples[j] + gap));
						}
					}
				}
			}
			gaps.insert(gap, tuples[i]);
		}
		
		if (i & 0x7fff) == 0 && tuples[i] > MAX_GAP
		{
			let limit = tuples[i] - MAX_GAP;
			gaps.retain(|_, &ft| { ft > limit });
			print!("{}/{}\r", i, tuples.len());
			io::stdout().flush().unwrap();
		}
	}
}
