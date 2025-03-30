use grainiac_core::Sampler;

#[test]
fn play_20_seconds() {
    let (mut sampler, _) = Sampler::new(44100.0);
    let mut data = vec![0.0; 44100 * 20];

    sampler.note_on(60);
    sampler.record(0);
    for sample in data.iter_mut() {
        let mut sample_l = *sample;
        let mut sample_r = sample.clone();
        sampler.render((&mut sample_l, &mut sample_r));
    }
}
