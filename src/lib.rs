use candid::{CandidType, Deserialize};
use ic_cdk_macros::*;
use sha2::{Digest, Sha256};

#[derive(Clone, CandidType, Deserialize)]
struct EngineState {
    dim: u32,
    step: u64,
    field: Vec<f64>,
    alpha: f64,
}

thread_local! {
    static STATE: std::cell::RefCell<Option<EngineState>> = std::cell::RefCell::new(None);
}

// ---------- Deterministic PRNG (simple xorshift) ----------
fn xorshift64(mut x: u64) -> u64 {
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

// For strict determinism across redeployments/canisters, keep this seed-based.
// (Later we can add an optional mode that mixes in canister id.)
fn seed_to_u64(seed: u64) -> u64 {
    seed
}

fn idx(dim: u32, x: u32, y: u32) -> usize {
    (y as usize) * (dim as usize) + (x as usize)
}

fn wrap(dim: u32, v: i64) -> u32 {
    let d = dim as i64;
    let mut r = v % d;
    if r < 0 {
        r += d;
    }
    r as u32
}

// ---------- Public API ----------
#[init]
fn init() {
    // no-op; explicit init via init_engine()
}

#[update]
fn init_engine(dim: u32, seed: u64, alpha: f64) {
    assert!(dim >= 4 && dim <= 512, "dim out of range");
    assert!(alpha > 0.0 && alpha <= 0.25, "alpha out of safe range");

    let mut s = seed_to_u64(seed);
    let n = (dim as usize) * (dim as usize);
    let mut field = Vec::with_capacity(n);

    for _ in 0..n {
        s = xorshift64(s);
        // map to [-1, 1] deterministically
        let u = (s as f64) / (u64::MAX as f64);
        field.push(u * 2.0 - 1.0);
    }

    let st = EngineState {
        dim,
        step: 0,
        field,
        alpha,
    };
    STATE.with(|cell| *cell.borrow_mut() = Some(st));
}

#[update]
fn tick(n: u32) {
    STATE.with(|cell| {
        let mut opt = cell.borrow_mut();
        let st = opt.as_mut().expect("engine not initialized");

        let dim = st.dim;
        let alpha = st.alpha;

        // double-buffer: next starts as a copy, then we overwrite each cell
        let mut next = st.field.clone();

        for _ in 0..n {
            for y in 0..dim {
                for x in 0..dim {
                    let c = st.field[idx(dim, x, y)];
                    let l = st.field[idx(dim, wrap(dim, x as i64 - 1), y)];
                    let r = st.field[idx(dim, wrap(dim, x as i64 + 1), y)];
                    let u = st.field[idx(dim, x, wrap(dim, y as i64 - 1))];
                    let d = st.field[idx(dim, x, wrap(dim, y as i64 + 1))];

                    // 4-neighbor Laplacian (periodic boundary)
                    let lap = (l + r + u + d) - 4.0 * c;
                    next[idx(dim, x, y)] = c + alpha * lap;
                }
            }

            // commit step
            st.field.clone_from(&next);
            st.step += 1;
        }
    });
}

#[query]
fn get_step() -> u64 {
    STATE.with(|cell| cell.borrow().as_ref().map(|s| s.step).unwrap_or(0))
}

#[query]
fn get_dim() -> u32 {
    STATE.with(|cell| cell.borrow().as_ref().map(|s| s.dim).unwrap_or(0))
}

#[query]
fn get_hash() -> Vec<u8> {
    STATE.with(|cell| {
        let st = cell.borrow();
        let s = st.as_ref().expect("engine not initialized");

        let mut h = Sha256::new();
        h.update(s.dim.to_le_bytes());
        h.update(s.step.to_le_bytes());
        h.update(s.alpha.to_le_bytes());

        // hash floats deterministically via bytes
        for v in &s.field {
            h.update(v.to_le_bytes());
        }

        h.finalize().to_vec()
    })
}

#[query]
fn get_field_slice(x0: u32, y0: u32, w: u32, h: u32) -> Vec<f64> {
    STATE.with(|cell| {
        let st = cell.borrow();
        let s = st.as_ref().expect("engine not initialized");
        let dim = s.dim;

        assert!(x0 < dim && y0 < dim, "start out of range");
        assert!(w >= 1 && h >= 1, "invalid size");
        assert!(x0 + w <= dim && y0 + h <= dim, "slice out of range");

        let mut out = Vec::with_capacity((w * h) as usize);
        for yy in y0..(y0 + h) {
            for xx in x0..(x0 + w) {
                out.push(s.field[idx(dim, xx, yy)]);
            }
        }
        out
    })
}
