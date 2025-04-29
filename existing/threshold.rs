
// use threshold_bls::{
//     schemes::bls12_381::G1Scheme as SigScheme, 
//     sig::{Scheme, ThresholdScheme, Share},
//     poly::Poly, poly::Idx
// };
// use rand::thread_rng;

// fn threshold_demo() {
//     // 1. Generate threshold key shares for n = 5, threshold t = 3
//     let n = 5;
//     let t = 3;
//     let mut rng = thread_rng();

//     // Create a random polynomial of degree t-1 representing the secret
//     let secret_poly = Poly::<SigScheme::Private>::new(t - 1);
//     // Generate n shares (points on the polynomial)
//     let shares: Vec<Share> = (0..n)
//         .map(|i| {
//             let eval = secret_poly.eval(i as Idx);
//             Share { index: eval.index, private: eval.value }
//         })
//         .collect();

//     // Public polynomial commitments, and the public (threshold) key
//     let pub_poly = secret_poly.commit();
//     let threshold_pk = pub_poly.public_key();

//     // 2. Sign a message with each share
//     let message = b"Epoch 1000 randomness";
//     let partial_sigs: Vec<_> = shares.iter()
//         .map(|share| SigScheme::partial_sign(share, message).unwrap())
//         .collect();

//     // 3. Verify each partial signature against the public polynomial
//     for sig in &partial_sigs {
//         SigScheme::partial_verify(&pub_poly, message, sig)
//             .expect("partial signature verification failed");
//     }

//     // 4. Aggregate t partials into a full signature
//     let full_sig = SigScheme::aggregate(t, &partial_sigs[..t]).unwrap();
//     // 5. Verify the full threshold signature with the threshold public key
//     SigScheme::verify(&threshold_pk, message, &full_sig)
//         .expect("threshold signature verification failed");

//     // If verification passes, `full_sig` is the unpredictable beacon for the epoch.
//     println!("Threshold signature valid.");
// }
