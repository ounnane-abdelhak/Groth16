# Groth16 — Zero-Knowledge Proof System in Rust

A from-scratch implementation of the Groth16 zk-SNARK proving system, built in Rust using the [arkworks](https://arkworks.rs) ecosystem for elliptic curve arithmetic over the BN254 curve.

---

## What is Groth16?

Groth16 is a **zero-knowledge succinct non-interactive argument of knowledge (zk-SNARK)** introduced by Jens Groth in 2016. It allows a **prover** to convince a **verifier** that they know a secret witness satisfying a given computation — without revealing any information about the witness itself.

### Key properties

- **Zero-knowledge** — the proof reveals nothing about the witness beyond the truth of the statement
- **Succinctness** — the proof consists of exactly 3 elliptic curve points (~200 bytes), regardless of circuit size
- **Non-interactive** — no back-and-forth between prover and verifier
- **Constant-time verification** — verification requires exactly 3 pairing operations, independent of circuit complexity

---

## How it works

The protocol proceeds in four stages:

### 1. Arithmetization — R1CS
The computation is encoded as a **Rank-1 Constraint System (R1CS)**. Every gate in the circuit becomes a constraint of the form:

```
(A · w) × (B · w) = (C · w)
```

where `w` is the witness vector containing all intermediate values of the computation. Constraints are represented sparsely as linear combinations over a finite field.

### 2. QAP Reduction
The R1CS constraint matrices are converted into a **Quadratic Arithmetic Program (QAP)** via Lagrange interpolation over the domain `{1, 2, ..., m}` where `m` is the number of constraints.

Each variable index `i` yields three polynomials `Aᵢ(x)`, `Bᵢ(x)`, `Cᵢ(x)` such that the R1CS is satisfied if and only if:

```
A(x) · B(x) - C(x) = H(x) · t(x)
```

where `t(x) = (x-1)(x-2)···(x-m)` is the vanishing polynomial and `H(x)` is the quotient polynomial computed by the prover.

### 3. Trusted Setup
A one-time ceremony samples toxic waste `(α, β, τ, γ, δ)` and encodes structured powers of `τ` into elliptic curve points over G1 and G2:

```
SRS_G1 = [G1, τ·G1, τ²·G1, ..., τⁿ·G1]
SRS_G2 = [G2, τ·G2, τ²·G2, ..., τⁿ·G2]
```

The QAP polynomials are evaluated at `τ` inside the curve to produce the **proving key** and **verifying key**. The toxic waste is destroyed after the ceremony — if it leaks, false proofs can be forged.

The proving key encodes:
- `[α]₁, [β]₁, [β]₂, [δ]₁` — blinding elements
- `aᵢ_query` — `[Aᵢ(τ)]₁` for all variables
- `bᵢ_g1_query`, `bᵢ_g2_query` — `[Bᵢ(τ)]₁` and `[Bᵢ(τ)]₂`
- `h_query` — `[τⁱ · t(τ)/δ]₁` for the quotient polynomial
- `l_query_pv` — private witness encodings `[(βAᵢ(τ) + αBᵢ(τ) + Cᵢ(τ))/δ]₁`

The verifying key encodes:
- `[α]₁, [β]₂, [γ]₂, [δ]₂`
- `gamma_abc_g1` — public witness encodings `[(βAᵢ(τ) + αBᵢ(τ) + Cᵢ(τ))/γ]₁`

### 4. Proving
Given the witness `w = (a₁, ..., aₘ)` and two fresh random blinding scalars `r, s`, the prover computes three curve points:

```
[A]₁ = [α]₁ + Σᵢ aᵢ·[Aᵢ(τ)]₁ + r·[δ]₁
[B]₂ = [β]₂ + Σᵢ aᵢ·[Bᵢ(τ)]₂ + s·[δ]₂
[C]₁ = Σᵢ₌ₗ₊₁ᵐ aᵢ·[Ψᵢ]₁ + [H(τ)·t(τ)]₁ + s·[A]₁ + r·[B]₁ - rs·[δ]₁
```

The proof `π = ([A]₁, [B]₂, [C]₁)` is published alongside the public inputs.

### 5. Verification
The verifier checks a single pairing equation:

```
e(A, B) = e(α, β) · e(Σᵢ₌₁ˡ aᵢ·Ψᵢ, γ) · e(C, δ)
```

This requires exactly **3 pairing computations** and runs in constant time regardless of circuit size. The Schwartz-Zippel lemma guarantees soundness — a false proof would require solving the discrete logarithm problem.

---

## Project Structure

```
src/
├── main.rs        — example circuit and entry point
├── field.rs       — field and curve type aliases (BN254)
├── poly.rs        — polynomial arithmetic (add, mul, eval, mod, interpolation)
├── r1cs.rs        — R1CS constraint system with sparse linear combinations
├── qap.rs         — QAP reduction via Lagrange interpolation
├── setup.rs       — trusted setup, proving key and verifying key generation
├── proof.rs       — prover
└── verify.rs      — verifier
```

---

## Dependencies

```toml
[dependencies]
ark-ff     = "0.5"   # finite field arithmetic
ark-ec     = "0.5"   # elliptic curve operations
ark-bn254  = "0.5"   # BN254 curve parameters
ark-std    = "0.5"   # utilities and RNG

```

---

## Example

The included example proves knowledge of `x = 3` satisfying `x³ + x + 5 = 35`:

```
witness = [1, x, x², x³, tmp, out]
        = [1, 3,  9, 27,  30,  35]

constraints:
  x  · x  = x²         (3  · 3  = 9)
  x² · x  = x³         (9  · 3  = 27)
  (x³ + x)  · 1 = tmp  (30 · 1  = 30)
  (tmp + 5) · 1 = out  (35 · 1  = 35)
```

```
is_satisfied: true
verify: true
```

---

## Security Notes

- This implementation is for **educational purposes only** and has not been audited
- The trusted setup uses `ark_std::test_rng()` — **not suitable for production**
- For production use, a proper multi-party computation (MPC) ceremony is required to generate the toxic waste securely

---

## References

- [Groth16 original paper](https://eprint.iacr.org/2016/260.pdf) — Jens Groth, *On the Size of Pairing-based Non-interactive Arguments*, 2016
- [arkworks documentation](https://docs.rs/ark-ec)
- [Why and How zk-SNARK Works](https://arxiv.org/abs/1906.07221) — Maksym Petkus
