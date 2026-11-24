use presskit_core::*;
#[test] fn flow(){
    let mut a=App::default();assert_eq!(a.scene(),Scene::Launcher);
    a.activate(Hit::Accept,0.);assert_eq!(a.effects(),vec![Effect::Fullscreen,Effect::Play(INTRO)]);
    a.media_complete();for _ in 0..74{a.tick();}assert_eq!(a.scene(),Scene::Welcome);
    a.activate(Hit::Videos,0.);for _ in 0..5{a.tick();}assert_eq!(a.scene(),Scene::Videos);
    a.activate(Hit::Trailer(3),0.);assert!(!a.playing);
    a.activate(Hit::Trailer(4),0.);for _ in 0..4{a.tick();}assert_eq!(a.scene(),Scene::Video);
    assert!(a.effects().contains(&Effect::Play(trailer(4).unwrap())));
    a.media_complete();for _ in 0..4{a.tick();}assert_eq!(a.scene(),Scene::Videos);
    a.activate(Hit::Screens,0.);for _ in 0..5{a.tick();}
    a.activate(Hit::Thumbnail(20),0.);for _ in 0..4{a.tick();}assert_eq!(a.scene(),Scene::Gallery);
    a.xml_loaded((1..=20).map(|i|format!("payload/screens/{i}.jpg")).collect());
    assert!(a.effects().contains(&Effect::Image("payload/screens/20.jpg".into())));
    a.activate(Hit::Previous,0.);assert_eq!(a.gallery.counter(),"19 / 20");
}
#[test] fn background_and_controls(){
    let mut b=Background::default();b.advance(42.);assert_eq!(b.frame,2);b.advance(1999.);assert_eq!(b.frame,2);b.advance(43.);assert_eq!(b.frame,3);
    let mut c=Controls::default();c.activity();for _ in 0..50{c.tick();}assert!(!c.visible);assert_eq!(c.target,620.);c.tick();assert_eq!(c.target,770.);
    assert_eq!(flash_time(59.6),"01:00");
}
#[test] fn hit_and_scale(){
    assert_eq!(hit_launcher(440.,410.),Hit::Accept);
    assert_eq!(hit_menu(Scene::Videos,900.,220.),Hit::Trailer(4));
    assert_eq!(hit_menu(Scene::Videos,900.,390.),Hit::None);
    assert_eq!(hit_menu(Scene::Screens,962.,444.),Hit::Thumbnail(20));
    assert_eq!(Stage::fit(2560.,1440.).point(1800.,440.),Some((900.,220.)));
}
#[test] fn enhanced_gallery_guard(){
    let mut g=Gallery::default();g.install(vec!["a".into(),"b".into()]);
    g.loaded=1;g.total_bytes=2;assert!(g.next().is_none());assert_eq!(g.index,0);
    g.loaded=2;assert_eq!(g.next().as_deref(),Some("b"));
}
