use bevy::{
    prelude::*,
    ui_widgets::{Activate, Button},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, set_up)
        .run();
}

fn set_up(mut commands: Commands) {
    commands.spawn(Camera2d);
    // 生成钢琴键盘场景
    commands.spawn_scene(keyboard(percent(80.0), percent(25.0), "column", 4));
}

fn keyboard(width: Val, height: Val, flow: &str, id: i8) -> impl Scene {
    let mut root = Node {
        width,
        height,
        display: Display::Grid,
        ..Default::default()
    };

    let mut white_plan = Node {
        display: Display::Flex,
        width: percent(100.0),
        height: percent(100.0),
        position_type: PositionType::Absolute, // 作为背景层
        ..Default::default()
    };

    let mut black_plan = Node {
        display: Display::Grid, // 不再是 Flex！
        width: percent(100.0),
        height: percent(100.0),
        position_type: PositionType::Absolute,
        ..Default::default()
    };

    match flow {
        "column" => {
            root.grid_template_columns = vec![RepeatedGridTrack::flex(7, 1.0)];
            white_plan.grid_column = GridPlacement::span(7);
            white_plan.flex_direction = FlexDirection::Row; // 水平键盘：白键横排
            black_plan.grid_column = GridPlacement::span(7);
            black_plan.grid_template_columns = vec![RepeatedGridTrack::flex(21, 1.0)];
        }
        "row" => {
            root.grid_template_rows = vec![RepeatedGridTrack::flex(7, 1.0)];
            white_plan.grid_row = GridPlacement::span(7);
            white_plan.flex_direction = FlexDirection::Column; // ✅ 垂直键盘：白键竖排
            black_plan.grid_row = GridPlacement::span(7); // ✅ 黑键层改为跨行
            black_plan.grid_template_rows = vec![RepeatedGridTrack::flex(21, 1.0)]; // ✅ 按行划分
        }
        _ => panic!("not has this type flow"),
    }

    bsn! {
        root
        Children [
            white_plan
            Children [
                @white_key(flow,KeyGroup::D,id)
                --
                @white_key(flow,KeyGroup::C,id)
                --
                @white_key(flow,KeyGroup::E,id)
                --
                @white_key(flow,KeyGroup::F,id)
                --
                @white_key(flow,KeyGroup::G,id)
                --
                @white_key(flow,KeyGroup::A,id)
                --
                @white_key(flow,KeyGroup::B,id)
            ]
            --
            // ✅ 黑键层：改为 Display::Grid，并划分为 21 列
            black_plan
            Children [
                // 黑键通过 start_span 精确定位
                @black_key(None,Some(percent(60.0)),flow,3,KeyGroup::CS,id)  // C#: 从第 4 个网格线开始（1-indexed），跨越 3 列
                --
                @black_key(None,Some(percent(60.0)),flow,6,KeyGroup::DS,id)  // D#
                --
                @black_key(None,Some(percent(60.0)),flow,12,KeyGroup::FS,id) // F#
                --
                @black_key(None,Some(percent(60.0)),flow,15,KeyGroup::GS,id) // G#
                --
                @black_key(None,Some(percent(60.0)),flow,18,KeyGroup::AS,id) // A#
            ]
        ]
    }
}

fn white_key(flow: &str, key: KeyGroup, id: i8) -> impl Scene {
    let mut node = Node {
        flex_grow: 1.0,
        height: percent(100.0),
        border: UiRect::all(percent(0.4)),
        margin: UiRect::all(percent(0.25)),
        ..Default::default()
    };
    match flow {
        "column" => {
            node.margin.top = Val::default();
            node.border.top = Val::default();
        }
        "row" => {
            node.margin.left = Val::default();
            node.border.left = Val::default();
        }
        _ => panic!("not has this type flow"),
    }
    bsn! {
        node
        key
        KeyGroupId(id)
        BackgroundColor(Color::WHITE)
        BorderColor::all(Srgba::rgb(0.8, 0.8, 0.8))
        Button
    }
}

/// 黑键：在 21 列网格中，从指定线开始，跨越 3 列
fn black_key(
    width: Option<Val>,
    height: Option<Val>,
    flow: &str,
    grid_column_start: i16,
    key: KeyGroup,
    id: i8,
) -> impl Scene {
    let mut node = Node {
        // 从 grid_column_start 开始，跨越 3 个网格单元
        height: height.unwrap_or_default(),
        width: width.unwrap_or_default(),
        border: UiRect::all(percent(2.5)),
        ..Default::default()
    };
    match flow {
        "column" => {
            node.grid_column = GridPlacement::start_span(grid_column_start, 2);
            node.border.top = Val::default();
        }
        "row" => {
            node.grid_row = GridPlacement::start_span(grid_column_start, 2);
            node.border.left = Val::default();
        }
        _ => panic!("not has this type flow"),
    }
    bsn! {
        node
        key
        KeyGroupId(id)
        BackgroundColor(Color::BLACK)
        BorderColor::all(Srgba::rgb( 0.2, 0.2, 0.2))
        Button
        on(test)
    }
}

#[derive(Debug, Component, Clone, PartialEq, Eq, Hash, Deref, DerefMut, Default)]
pub struct KeyGroupId(pub i8);

#[derive(Debug, Component, Clone, PartialEq, Eq, Hash)]
pub enum KeyGroup {
    C,
    CS,
    D,
    DS,
    E,
    F,
    FS,
    G,
    GS,
    A,
    AS,
    B,
}

pub fn test(trigger: On<Activate>, query: Query<(&KeyGroup, &KeyGroupId)>) {
    if let Ok((key, id)) = query.get(trigger.entity) {
        println!("点击了键: {:?}, 所属组: {}", key, id.0);
    }
}
