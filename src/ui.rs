use std::cmp;

use bevy::{
    color::palettes::css::{BLACK, GREEN, WHITE},
    ecs::query::QueryData,
    prelude::*,
};

#[derive(Event, Default, Debug)]
pub struct UiInputsEvent {
    pub energy_level: String,
}

#[derive(Resource)]
pub struct UiInputEntities {
    pub energy_level: Entity,
}

#[derive(Component, Debug, Clone, Copy)]
pub struct EnergyLevel(pub u32);

#[derive(Component, Default, QueryData)]
pub struct EnergyLabelMarker;
#[derive(Component, Default)]
pub struct EnergyLevelPlusMarker;
#[derive(Component, Default)]
pub struct EnergyLevelMinusMarker;

pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    let root = commands.spawn(NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            top: Val::Px(0.0),
            right: Val::Px(0.0),
            width: Val::Px(130.0),
            height: Val::Percent(100.0),
            ..default()
        },
        background_color: BackgroundColor(Color::BLACK),
        ..default()
    });

    let root_id = root.id();

    add_label(&mut commands, root_id, &font, "Energy level:");

    let row = NodeBundle {
        style: Style {
            position_type: PositionType::Relative,
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.0),
            height: Val::Px(30.0),
            margin: UiRect {
                top: Val::Px(10.0),
                ..default()
            },
            ..default()
        },
        background_color: BackgroundColor(Color::BLACK),
        ..default()
    };
    let row_id = commands.spawn(row).id();
    commands.entity(root_id).push_children(&[row_id]);

    let energy_level_value_label = generate_label(&font, "1");
    let spawned_energy_level_value_label = commands
        .spawn((EnergyLabelMarker, energy_level_value_label))
        .id();
    commands
        .entity(root_id)
        .push_children(&[spawned_energy_level_value_label]);

    add_square_button(&mut commands, row_id, &font, "-", EnergyLevelMinusMarker);
    add_square_button(&mut commands, row_id, &font, "+", EnergyLevelPlusMarker);

    commands.insert_resource(UiInputEntities {
        energy_level: spawned_energy_level_value_label,
    });

    commands.spawn(EnergyLevel(1));
}

pub fn generate_label(font: &Handle<Font>, label: &str) -> TextBundle {
    TextBundle {
        style: Style {
            position_type: PositionType::Relative,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            width: Val::Percent(100.0),
            height: Val::Auto,
            ..default()
        },
        text: Text::from_section(
            label.to_string(),
            TextStyle {
                font: font.clone(),
                font_size: 14.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    }
}

pub fn add_label(
    commands: &mut Commands,
    root_id: Entity,
    font: &Handle<Font>,
    label: &str,
) -> Entity {
    let label = generate_label(font, label);
    let spawned_label = commands.spawn(label).id();
    commands.entity(root_id).push_children(&[spawned_label]);
    spawned_label
}

pub fn add_square_button<T>(
    commands: &mut Commands,
    root_id: Entity,
    font: &Handle<Font>,
    label: &str,
    marker: T,
) where
    T: Component,
{
    let button = commands
        .spawn((
            marker,
            ButtonBundle {
                style: Style {
                    top: Val::Px(0.0),
                    width: Val::Px(30.0),
                    height: Val::Px(30.0),
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    label.to_string(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 14.0,
                        color: WHITE.into(),
                    }
                    .clone(),
                ),
                ..Default::default()
            });
        })
        .id();
    commands.entity(root_id).push_children(&[button]);
}

/// processes the gui events
// TODO error handling (show on ui)
#[allow(clippy::too_many_arguments)]
pub fn listen_ui_inputs(
    mut events: EventReader<UiInputsEvent>,
    mut commands: Commands,
    energy_level_query: Query<Entity, With<EnergyLevel>>,
) {
    for input in events.read() {
        match parse_i32(&input.energy_level) {
            Ok(i) => {
                despawn_all_entities(&mut commands, &energy_level_query);
                commands.spawn(EnergyLevel(i));
            }
            Err(err) => println!("error: {}", err),
        }
    }
}

pub fn parse_i32(str: &str) -> Result<u32, String> {
    let f = str.parse::<u32>();
    match f {
        Ok(i) => Ok(i),
        Err(e) => Err(format!("Failed to parse u32: {}", e)),
    }
}

pub fn despawn_all_entities<T>(commands: &mut Commands, query: &Query<Entity, With<T>>)
where
    T: Component,
{
    for e in query.iter() {
        let entity = commands.entity(e);
        entity.despawn_recursive();
    }
}

pub fn despawn_all_entities_tu<T, U>(
    commands: &mut Commands,
    query: &Query<Entity, (With<T>, With<U>)>,
) where
    T: Component,
    U: Component,
{
    for e in query.iter() {
        let entity = commands.entity(e);
        entity.despawn_recursive();
    }
}

pub fn plus_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<EnergyLevelPlusMarker>),
    >,
    mut my_events: EventWriter<PlusMinusInputEvent>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        plus_minus_button_handler(
            (interaction, &mut color, &mut border_color),
            &mut my_events,
            PlusMinusInput::Plus,
        );
    }
}

pub fn minus_button_handler(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut BorderColor),
        (Changed<Interaction>, With<EnergyLevelMinusMarker>),
    >,
    mut my_events: EventWriter<PlusMinusInputEvent>,
) {
    for (interaction, mut color, mut border_color) in &mut interaction_query {
        plus_minus_button_handler(
            (interaction, &mut color, &mut border_color),
            &mut my_events,
            PlusMinusInput::Minus,
        );
    }
}

fn plus_minus_button_handler(
    interaction: (&Interaction, &mut BackgroundColor, &mut BorderColor),
    my_events: &mut EventWriter<PlusMinusInputEvent>,
    plus_minus: PlusMinusInput,
) {
    let (interaction, color, border_color) = interaction;
    match *interaction {
        Interaction::Pressed => {
            *color = GREEN.into();
            border_color.0 = GREEN.into();
            println!("sending plus minus event: {:?}", plus_minus);
            my_events.send(PlusMinusInputEvent { plus_minus });
        }
        Interaction::Hovered => {}
        Interaction::None => {
            *color = BLACK.into();
            border_color.0 = BLACK.into();
        }
    }
}

/// processes the gui events
// TODO error handling (show on ui)
#[allow(clippy::too_many_arguments)]
pub fn listen_energy_level_ui_inputs(
    mut events: EventReader<PlusMinusInputEvent>,
    mut commands: Commands,
    mut energy_level_query: Query<&EnergyLevel>,
    energy_level_entity_query: Query<Entity, With<EnergyLevel>>,
) {
    for input in events.read() {
        for e in energy_level_query.iter_mut() {
            // println!("got energy level: {:?}", e);
            let current = e.0;
            let increment: i32 = match input.plus_minus {
                PlusMinusInput::Plus => 1,
                PlusMinusInput::Minus => -1,
            };
            let new_i = current as i32 + increment;
            let new = cmp::max(0, new_i) as u32; // pressing "-" at 0 stays at 0

            despawn_all_entities(&mut commands, &energy_level_entity_query);
            let energy_level = EnergyLevel(new);
            commands.spawn(energy_level);
        }
    }
}

pub fn update_energy_level_label(
    mut commands: Commands,
    energy_level_query: Query<&EnergyLevel>,
    input_entities: Res<UiInputEntities>,
    mut label_query: Query<(Entity, &mut Text), With<EnergyLabelMarker>>,
) {
    for e in energy_level_query.iter() {
        let a = commands.entity(input_entities.energy_level);

        for (entity, mut text) in label_query.iter_mut() {
            if entity == a.id() {
                text.sections[0].value = e.0.to_string();
            }
        }
    }
}

#[derive(Debug, Default, Clone, Copy, Resource)]
pub enum PlusMinusInput {
    #[default]
    Plus,
    Minus,
}

#[derive(Event, Default, Debug)]
pub struct PlusMinusInputEvent {
    pub plus_minus: PlusMinusInput,
}
