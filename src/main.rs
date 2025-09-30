use std::{fs, path::Path, path::PathBuf, process::exit, time::Duration};
use rand::prelude::IndexedRandom;
use bevy::prelude::*;

#[derive(Component)]
struct CurrentFileText;

#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct CurrentAudioPlayer;

#[derive(Component)]
struct PlayPauseButton;

#[derive(Component)]
struct StopButton;

#[derive(Component)]
struct StartButton;

#[derive(Component)]
struct AnswerInput;

#[derive(Component)]
struct SubmitButton;

#[derive(Resource)]
struct AudioState {
    state: PlayState,
    pause_timer: Timer,
    play_start_timer: Timer,
    user_paused: bool,
    current_file: Option<String>,
    current_file_path: Option<PathBuf>,
    user_answer: String,
    correct_count: u32,
    wrong_count: u32,
    was_correct: bool,
    repeat_count: u32,
    repeat_pause_timer: Timer,
}

#[derive(PartialEq, Clone)]
enum PlayState {
    Playing,
    Pausing,
    WaitingForAnswer,
    ReadyToPlay,
    Stopped,
    RepeatPlaying,
    RepeatPausing,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            state: PlayState::ReadyToPlay,
            pause_timer: Timer::new(Duration::from_secs(2), TimerMode::Once),
            play_start_timer: Timer::new(Duration::from_millis(500), TimerMode::Once),
            user_paused: false,
            current_file: None,
            current_file_path: None,
            user_answer: String::new(),
            correct_count: 0,
            wrong_count: 0,
            was_correct: true,
            repeat_count: 0,
            repeat_pause_timer: Timer::new(Duration::from_millis(800), TimerMode::Once),
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "CW Listen Trainer".to_string(),
                resolution: (800.0, 400.0).into(),
                ..default()
            }),
            ..default()
        }))
        .init_resource::<AudioState>()
        .add_systems(Startup, setup_ui)
        .add_systems(Update, (audio_player_system, button_system, text_input_system, keyboard_input_system, update_score_display))
        .run();
}

fn setup_ui(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    commands.spawn((
        Text::new("Bereit..."),
        TextFont {
            font_size: 24.0,
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(JustifyText::Left),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(50.0),
            right: Val::Px(50.0),
            top: Val::Px(20.0),
            height: Val::Px(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            ..default()
        },
        CurrentFileText,
    ));
    
    // Score Anzeige rechts oben
    commands.spawn((
        Text::new("Richtig: 0 | Falsch: 0"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(Color::srgb(0.8, 0.8, 0.8)),
        TextLayout::new_with_justify(JustifyText::Right),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(50.0),
            top: Val::Px(20.0),
            ..default()
        },
        ScoreText,
    ));
    
    commands.spawn((
        Node {
            width: Val::Px(300.0),
            height: Val::Px(50.0),
            position_type: PositionType::Absolute,
            left: Val::Px(250.0),
            top: Val::Px(140.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
        BorderColor(Color::srgb(0.5, 0.5, 0.5)),
        Visibility::Hidden,
        AnswerInput,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
    
    commands.spawn((
        Button,
        Node {
            width: Val::Px(150.0),
            height: Val::Px(50.0),
            position_type: PositionType::Absolute,
            left: Val::Px(325.0),
            top: Val::Px(210.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.3, 0.7, 0.3)),
        Visibility::Hidden,
        SubmitButton,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Bestaetigen"),
            TextFont {
                font_size: 20.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
    
    commands.spawn((
        Button,
        Node {
            width: Val::Px(150.0),
            height: Val::Px(50.0),
            position_type: PositionType::Absolute,
            left: Val::Px(80.0),
            bottom: Val::Px(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.5, 0.8)),
        StartButton,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Start"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
    
    commands.spawn((
        Button,
        Node {
            width: Val::Px(150.0),
            height: Val::Px(50.0),
            position_type: PositionType::Absolute,
            left: Val::Px(250.0),
            bottom: Val::Px(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.2, 0.6, 0.2)),
        PlayPauseButton,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Pause"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
    
    commands.spawn((
        Button,
        Node {
            width: Val::Px(150.0),
            height: Val::Px(50.0),
            position_type: PositionType::Absolute,
            left: Val::Px(420.0),
            bottom: Val::Px(30.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::srgb(0.8, 0.2, 0.2)),
        StopButton,
    ))
    .with_children(|parent| {
        parent.spawn((
            Text::new("Stopp"),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    });
}

fn button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, Option<&PlayPauseButton>, Option<&StopButton>, Option<&StartButton>, Option<&SubmitButton>),
        Changed<Interaction>,
    >,
    mut audio_state: ResMut<AudioState>,
    audio_query: Query<&AudioSink, With<CurrentAudioPlayer>>,
    button_query: Query<&Children, With<PlayPauseButton>>,
    mut status_text_query: Query<&mut Text, With<CurrentFileText>>,
    mut button_text_query: Query<&mut Text, Without<CurrentFileText>>,
) {
    for (interaction, mut color, play_pause, stop, start, submit) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                if start.is_some() {
                    if audio_state.state == PlayState::Stopped {
                        audio_state.state = PlayState::ReadyToPlay;
                        audio_state.user_paused = false;
                    }
                } else if submit.is_some() {
                    if audio_state.state == PlayState::WaitingForAnswer && !audio_state.user_answer.is_empty() {
                        if let Some(correct_answer) = &audio_state.current_file {
                            let correct_answer = correct_answer.clone();
                            let user_answer = audio_state.user_answer.trim().to_lowercase();
                            let correct = correct_answer.to_lowercase();
                            let is_correct = user_answer == correct;
                            
                            // Score aktualisieren
                            if is_correct {
                                audio_state.correct_count += 1;
                                audio_state.was_correct = true;
                                
                                for mut text in status_text_query.iter_mut() {
                                    **text = format!("{}", correct_answer);
                                }
                                
                                audio_state.user_answer.clear();
                                audio_state.pause_timer = Timer::new(Duration::from_secs(2), TimerMode::Once);
                                audio_state.pause_timer.reset();
                                audio_state.state = PlayState::Pausing;
                            } else {
                                audio_state.wrong_count += 1;
                                audio_state.was_correct = false;
                                
                                for mut text in status_text_query.iter_mut() {
                                    **text = format!("Erwartet: {}\nDeine Antwort: {}\n\nWiederhole 3x...", 
                                        correct_answer, audio_state.user_answer);
                                }
                                
                                audio_state.user_answer.clear();
                                audio_state.repeat_count = 0;
                                audio_state.repeat_pause_timer.reset();
                                audio_state.state = PlayState::RepeatPausing;
                            }
                            
                            println!("Antwort: {} | Richtig: {} | Korrekt: {}", user_answer, correct, is_correct);
                        }
                    }
                } else if play_pause.is_some() {
                    audio_state.user_paused = !audio_state.user_paused;
                    
                    for sink in audio_query.iter() {
                        if audio_state.user_paused {
                            sink.pause();
                        } else {
                            sink.play();
                        }
                    }
                    
                    if let Ok(children) = button_query.single() {
                        for child in children.iter() {
                            if let Ok(mut text) = button_text_query.get_mut(child) {
                                **text = if audio_state.user_paused {
                                    "Weiter".to_string()
                                } else {
                                    "Pause".to_string()
                                };
                            }
                        }
                    }
                } else if stop.is_some() {
                    audio_state.state = PlayState::Stopped;
                    audio_state.user_paused = false;
                }
            }
            Interaction::Hovered => {
                *color = if submit.is_some() {
                    Color::srgb(0.35, 0.8, 0.35).into()
                } else if play_pause.is_some() {
                    Color::srgb(0.25, 0.7, 0.25).into()
                } else if stop.is_some() {
                    Color::srgb(0.9, 0.3, 0.3).into()
                } else {
                    Color::srgb(0.3, 0.6, 0.9).into()
                };
            }
            Interaction::None => {
                *color = if submit.is_some() {
                    Color::srgb(0.3, 0.7, 0.3).into()
                } else if play_pause.is_some() {
                    Color::srgb(0.2, 0.6, 0.2).into()
                } else if stop.is_some() {
                    Color::srgb(0.8, 0.2, 0.2).into()
                } else {
                    Color::srgb(0.2, 0.5, 0.8).into()
                };
            }
        }
    }
}

fn keyboard_input_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut audio_state: ResMut<AudioState>,
    input_query: Query<&Children, With<AnswerInput>>,
    mut status_text_query: Query<&mut Text, With<CurrentFileText>>,
    mut all_text_query: Query<&mut Text, Without<CurrentFileText>>,
) {
    if audio_state.state != PlayState::WaitingForAnswer {
        return;
    }
    
    if keys.just_pressed(KeyCode::Backspace) {
        audio_state.user_answer.pop();
        update_input_text(&input_query, &mut all_text_query, &audio_state.user_answer);
    }
    
    if keys.just_pressed(KeyCode::Enter) && !audio_state.user_answer.is_empty() {
        check_answer(&mut audio_state, &mut status_text_query);
        return;
    }
    
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    
    let character = if keys.just_pressed(KeyCode::KeyA) { Some('a') }
    else if keys.just_pressed(KeyCode::KeyB) { Some('b') }
    else if keys.just_pressed(KeyCode::KeyC) { Some('c') }
    else if keys.just_pressed(KeyCode::KeyD) { Some('d') }
    else if keys.just_pressed(KeyCode::KeyE) { Some('e') }
    else if keys.just_pressed(KeyCode::KeyF) { Some('f') }
    else if keys.just_pressed(KeyCode::KeyG) { Some('g') }
    else if keys.just_pressed(KeyCode::KeyH) { Some('h') }
    else if keys.just_pressed(KeyCode::KeyI) { Some('i') }
    else if keys.just_pressed(KeyCode::KeyJ) { Some('j') }
    else if keys.just_pressed(KeyCode::KeyK) { Some('k') }
    else if keys.just_pressed(KeyCode::KeyL) { Some('l') }
    else if keys.just_pressed(KeyCode::KeyM) { Some('m') }
    else if keys.just_pressed(KeyCode::KeyN) { Some('n') }
    else if keys.just_pressed(KeyCode::KeyO) { Some('o') }
    else if keys.just_pressed(KeyCode::KeyP) { Some('p') }
    else if keys.just_pressed(KeyCode::KeyQ) { Some('q') }
    else if keys.just_pressed(KeyCode::KeyR) { Some('r') }
    else if keys.just_pressed(KeyCode::KeyS) { Some('s') }
    else if keys.just_pressed(KeyCode::KeyT) { Some('t') }
    else if keys.just_pressed(KeyCode::KeyU) { Some('u') }
    else if keys.just_pressed(KeyCode::KeyV) { Some('v') }
    else if keys.just_pressed(KeyCode::KeyW) { Some('w') }
    else if keys.just_pressed(KeyCode::KeyX) { Some('x') }
    else if keys.just_pressed(KeyCode::KeyY) { Some('y') }
    else if keys.just_pressed(KeyCode::KeyZ) { Some('z') }
    else if keys.just_pressed(KeyCode::Digit0) { Some('0') }
    else if keys.just_pressed(KeyCode::Digit1) { Some('1') }
    else if keys.just_pressed(KeyCode::Digit2) { Some('2') }
    else if keys.just_pressed(KeyCode::Digit3) { Some('3') }
    else if keys.just_pressed(KeyCode::Digit4) { Some('4') }
    else if keys.just_pressed(KeyCode::Digit5) { Some('5') }
    else if keys.just_pressed(KeyCode::Digit6) { Some('6') }
    else if keys.just_pressed(KeyCode::Digit7) { Some('7') }
    else if keys.just_pressed(KeyCode::Digit8) { Some('8') }
    else if keys.just_pressed(KeyCode::Digit9) { Some('9') }
    else if keys.just_pressed(KeyCode::Minus) { Some('-') }
    else if keys.just_pressed(KeyCode::Space) { Some(' ') }
    else if keys.just_pressed(KeyCode::Semicolon) { Some('ö') }
    else if keys.just_pressed(KeyCode::Quote) { Some('ä') }
    else if keys.just_pressed(KeyCode::BracketLeft) { Some('ü') }
    else { None };
    
    if let Some(ch) = character {
        let final_char = if shift && ch.is_alphabetic() {
            ch.to_uppercase().next().unwrap()
        } else {
            ch
        };
        
        audio_state.user_answer.push(final_char);
        update_input_text(&input_query, &mut all_text_query, &audio_state.user_answer);
    }
}

fn check_answer(
    audio_state: &mut ResMut<AudioState>,
    text_query: &mut Query<&mut Text, With<CurrentFileText>>,
) {
    if let Some(correct_answer) = &audio_state.current_file {
        let correct_answer = correct_answer.clone();
        let user_answer = audio_state.user_answer.trim().to_lowercase();
        let correct = correct_answer.to_lowercase();
        
        let is_correct = user_answer == correct;
        
        // Score aktualisieren
        if is_correct {
            audio_state.correct_count += 1;
            audio_state.was_correct = true;
            
            for mut text in text_query.iter_mut() {
                **text = format!("{}", correct_answer);
            }
            
            audio_state.user_answer.clear();
            audio_state.pause_timer = Timer::new(Duration::from_secs(2), TimerMode::Once);
            audio_state.pause_timer.reset();
            audio_state.state = PlayState::Pausing;
        } else {
            audio_state.wrong_count += 1;
            audio_state.was_correct = false;
            
            for mut text in text_query.iter_mut() {
                **text = format!("Erwartet: {}\nDeine Antwort: {}\n\nWiederhole 3x...", 
                    correct_answer, audio_state.user_answer);
            }
            
            audio_state.user_answer.clear();
            audio_state.repeat_count = 0;
            audio_state.repeat_pause_timer.reset();
            audio_state.state = PlayState::RepeatPausing;
        }
        
        println!("Antwort: {} | Richtig: {} | Korrekt: {}", user_answer, correct, is_correct);
    }
}

fn update_input_text(
    input_query: &Query<&Children, With<AnswerInput>>,
    text_query: &mut Query<&mut Text, Without<CurrentFileText>>,
    answer: &str,
) {
    if let Ok(children) = input_query.single() {
        for child in children.iter() {
            if let Ok(mut text) = text_query.get_mut(child) {
                **text = answer.to_string();
            }
        }
    }
}

fn text_input_system(
    mut input_query: Query<&mut Visibility, With<AnswerInput>>,
    mut submit_query: Query<&mut Visibility, (With<SubmitButton>, Without<AnswerInput>)>,
    audio_state: Res<AudioState>,
) {
    let should_show = audio_state.state == PlayState::WaitingForAnswer;
    
    for mut visibility in input_query.iter_mut() {
        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    
    for mut visibility in submit_query.iter_mut() {
        *visibility = if should_show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn update_score_display(
    audio_state: Res<AudioState>,
    mut score_query: Query<&mut Text, With<ScoreText>>,
) {
    if audio_state.is_changed() {
        for mut text in score_query.iter_mut() {
            **text = format!("Richtig: {} | Falsch: {}", 
                audio_state.correct_count, audio_state.wrong_count);
        }
    }
}

fn getsoundfile() -> (PathBuf, String) {
    let soundpath = "assets/sounds";
    let path = Path::new(soundpath);
    
    if !path.exists() {
        eprintln!("Fehler: Verzeichnis nicht gefunden: {}", soundpath);
        exit(99);
    }
 
    let mp3_files: Vec<_> = fs::read_dir(soundpath)
        .expect("Kann Verzeichnis nicht lesen")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path()
                .extension()
                .map_or(false, |ext| ext == "mp3" || ext == "ogg" || ext == "wav")
        })
        .collect();
    
    if mp3_files.is_empty() {
        eprintln!("Fehler: Keine Audio-Dateien in {} gefunden!", soundpath);
        exit(98);
    }
    
    let mut rng = rand::rng();
    let random_file = mp3_files.choose(&mut rng).unwrap();
    let full_path = random_file.path();
    
    let filename_without_ext = full_path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("Unbekannt")
        .to_string();
    
    let asset_path = full_path.strip_prefix("assets/").unwrap().to_owned();
    
    (asset_path, filename_without_ext)
}

fn audio_player_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut text_query: Query<&mut Text, With<CurrentFileText>>,
    audio_query: Query<(Entity, &AudioSink), With<CurrentAudioPlayer>>,
    mut audio_state: ResMut<AudioState>,
    time: Res<Time>,
) {
    if audio_state.state == PlayState::Stopped {
        for (entity, _) in audio_query.iter() {
            commands.entity(entity).despawn();
        }
        
        for mut text in text_query.iter_mut() {
            **text = "Gestoppt".to_string();
        }
        return;
    }
    
    if audio_state.user_paused {
        return;
    }
    
    match audio_state.state {
        PlayState::ReadyToPlay => {
            for (entity, _) in audio_query.iter() {
                commands.entity(entity).despawn();
            }
            
            let (randsoundfile, filename_without_ext) = getsoundfile();
            let audio_handle = asset_server.load(&*randsoundfile);
            commands.spawn((
                AudioPlayer::new(audio_handle),
                CurrentAudioPlayer,
            ));
            
            println!("Spiele: {} (Antwort: {})", randsoundfile.display(), filename_without_ext);
            
            for mut text in text_query.iter_mut() {
                **text = "Hoere zu...".to_string();
            }
            
            audio_state.current_file = Some(filename_without_ext);
            audio_state.current_file_path = Some(randsoundfile);
            audio_state.play_start_timer.reset();
            audio_state.state = PlayState::Playing;
        }
        PlayState::Playing => {
            audio_state.play_start_timer.tick(time.delta());
            
            if !audio_state.play_start_timer.finished() {
                return;
            }
            
            let is_playing = audio_query.iter().any(|(_, sink)| !sink.empty());
            
            if !is_playing {
                for (entity, _) in audio_query.iter() {
                    commands.entity(entity).despawn();
                }
                
                println!("Sound fertig, warte auf Antwort...");
                audio_state.state = PlayState::WaitingForAnswer;
                audio_state.user_answer.clear();
                
                for mut text in text_query.iter_mut() {
                    **text = "Welches Zeichen haben Sie gehört ?".to_string();
                }
            }
        }
        PlayState::WaitingForAnswer => {
        }
        PlayState::Pausing => {
            audio_state.pause_timer.tick(time.delta());
            
            if audio_state.pause_timer.fraction() < 0.3 {
            } else {
                for mut text in text_query.iter_mut() {
                    **text = "Pause...".to_string();
                }
            }
            
            if audio_state.pause_timer.finished() {
                println!("Pause vorbei, spiele naechsten Sound...");
                audio_state.state = PlayState::ReadyToPlay;
            }
        }
        PlayState::RepeatPausing => {
            audio_state.repeat_pause_timer.tick(time.delta());
            
            if audio_state.repeat_pause_timer.finished() {
                if audio_state.repeat_count < 3 {
                    // Starte nächste Wiederholung
                    for (entity, _) in audio_query.iter() {
                        commands.entity(entity).despawn();
                    }
                    
                    if let Some(file_path) = &audio_state.current_file_path {
                        let audio_handle = asset_server.load(&**file_path);
                        commands.spawn((
                            AudioPlayer::new(audio_handle),
                            CurrentAudioPlayer,
                        ));
                        
                        audio_state.repeat_count += 1;
                        println!("Wiederholung {}/3", audio_state.repeat_count);
                        
                        if let Some(correct_answer) = &audio_state.current_file {
                            for mut text in text_query.iter_mut() {
                                **text = format!("Loesung: {}\n\nWiederholung {}/3", 
                                    correct_answer, audio_state.repeat_count);
                            }
                        }
                        
                        audio_state.play_start_timer.reset();
                        audio_state.state = PlayState::RepeatPlaying;
                    }
                } else {
                    println!("Wiederholungen beendet, naechster Sound...");
                    audio_state.pause_timer = Timer::new(Duration::from_secs(2), TimerMode::Once);
                    audio_state.pause_timer.reset();
                    audio_state.state = PlayState::Pausing;
                }
            }
        }
        PlayState::RepeatPlaying => {
            audio_state.play_start_timer.tick(time.delta());
            
            // Prüfe zuerst ob bereits ein Sound spielt
            if !audio_state.play_start_timer.finished() {
                return;
            }
            
            let is_playing = audio_query.iter().any(|(_, sink)| !sink.empty());
            
            if !is_playing {
                // Sound ist fertig, starte Pause vor nächster Wiederholung
                for (entity, _) in audio_query.iter() {
                    commands.entity(entity).despawn();
                }
                
                audio_state.repeat_pause_timer = Timer::new(Duration::from_millis(800), TimerMode::Once);
                audio_state.repeat_pause_timer.reset();
                audio_state.state = PlayState::RepeatPausing;
            }
        }
        PlayState::Stopped => {
        }
    }
}
