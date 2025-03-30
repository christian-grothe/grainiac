use grainiac_core::Sampler;

#[test]
fn test_render() {
    let (mut sampler, _) = Sampler::new(48000.0);
    let mut sample_l = 0.0;
    let mut sample_r = 0.0;

    sampler.render((&mut sample_l, &mut sample_r));
}

#[test]
fn test_record() {
    let (mut sampler, _) = Sampler::new(48000.0);
    let mut sample_l = 1.0;
    let mut sample_r = 1.0;

    sampler.note_on(60);
    sampler.render((&mut sample_l, &mut sample_r));
}

#[test]
fn play_20_seconds() {
    let (mut sampler, _) = Sampler::new(44100.0);
    let mut data = vec![0.0; 44100 * 20];

    sampler.note_on(60);
    for sample in data.iter_mut() {
        let mut sample_l = *sample;
        let mut sample_r = sample.clone();
        sampler.render((&mut sample_l, &mut sample_r));
    }
}
