#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    SpawnGrass,
    SpawnShrub,
    SpawnTree,
    SpawnMushroom,
    SpawnInsect,
    SpawnFish,
    AddSoilEnergy,
    AddMoisture,
    Kill,
}

impl Tool {
    pub fn label(&self) -> &'static str {
        match self {
            Tool::Select =>        "🔍 Выбрать",
            Tool::SpawnGrass =>    "🌿 Посадить Траву",
            Tool::SpawnShrub =>    "🌳 Посадить Кустарник",
            Tool::SpawnTree =>     "🌲 Посадить Дерево",
            Tool::SpawnMushroom => "🍄 Посадить Гриб",
            Tool::SpawnInsect =>   "🐛 Создать Насекомое",
            Tool::SpawnFish =>     "🐟 Создать Рыбу",
            Tool::AddSoilEnergy => "⚡ Удобрить почву",
            Tool::AddMoisture =>   "💧 Увлажнить почву",
            Tool::Kill =>          "❌ Очистить зону",
        }
    }
}
