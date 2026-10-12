use thiserror::Error;

#[derive(Error, Debug)]
#[error("error writing packets")]
pub enum PacketWriteError {
    #[error("failed to convert type")]
    TypeConversion(#[from] std::num::TryFromIntError),
}

#[derive(Debug, PartialEq)]
pub enum OutgoingPacket {
    Status(ClientStatusPacket),
    Login(ClientLoginPacket),
    Config(ClientConfigPacket),
    Play(ClientPlayPacket),
}

#[derive(Debug, PartialEq)]
pub enum ClientStatusPacket {
    StatusResponse,
    PongResponse,
}

#[repr(u8)]
pub enum ClientStatusPacketId {
    StatusResponse = 0x00,
    PongResponse = 0x01,
}

impl TryFrom<u8> for ClientStatusPacketId {
    type Error = u8;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Self::StatusResponse),
            0x01 => Ok(Self::PongResponse),
            other => Err(other),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ClientLoginPacket {}

#[repr(u8)]
pub enum ClientLoginPacketId {
    DisconnectLogin = 0x00,
    EncryptionRequest = 0x01,
    LoginSuccess = 0x02,
    SetCompression = 0x03,
    LoginPluginRequest = 0x04,
    CookieRequest = 0x05,
}

#[derive(Debug, PartialEq)]
pub enum ClientConfigPacket {}

#[repr(u8)]
pub enum ClientConfigPacketId {
    CookieRequest = 0x00,
    PluginMessageClientbound = 0x01,
    Disconnect = 0x02,
    FinishConfig = 0x03,
    KeepAliveClientbound = 0x04,
    Ping = 0x05,
    ResetChat = 0x06,
    RegistryData = 0x07,
    RemoveResourcePack = 0x08,
    AddResourcePack = 0x09,
    PostEffects = 0x0A,
    StoreCookie = 0x0B,
    Transfer = 0x0C,
    FeatureFlags = 0x0D,
    UpdateTags = 0x0E,
    KnownPacksClientbound = 0x0F,
    CustomReportDetails = 0x10,
    ServerLinks = 0x11,
    ClearDialog = 0x12,
    ShowDialogConfig = 0x13,
    CodeOfConduct = 0x14,
}

#[derive(Debug, PartialEq)]
pub enum ClientPlayPacket {}

#[repr(u8)]
pub enum ClientPlayPacketId {
    BundleDelimiter = 0x00,
    SpawnEntity = 0x01,
    EntityAnimation = 0x02,
    AwardStatistics = 0x03,
    AcknowledgeBlockChange = 0x04,
    SetBlockDestroyStage = 0x05,
    BlockEntityData = 0x06,
    BlockAction = 0x07,
    BlockUpdate = 0x08,
    BossBar = 0x09,
    ChangeDifficulty = 0x0A,
    ChunkBatchFinished = 0x0B,
    ChunkBatchStart = 0x0C,
    ChunkBiomes = 0x0D,
    ClearTitles = 0x0E,
    CommandSuggestionsResponse = 0x0F,
    Commands = 0x10,
    CloseContainer = 0x11,
    SetContainerContent = 0x12,
    SetContainerProperty = 0x13,
    SetContainerSlot = 0x14,
    CookieRequest = 0x15,
    SetCooldown = 0x16,
    ChatSuggestions = 0x17,
    PluginMessageClientbound = 0x18,
    DamageEvent = 0x19,
    DebugBlockValue = 0x1A,
    DebugChunkValue = 0x1B,
    DebugEntityValue = 0x1C,
    DebugEvent = 0x1D,
    DebugSample = 0x1E,
    DeleteMessage = 0x1F,
    Disconnect = 0x20,
    DisguisedChatMessage = 0x21,
    EntityEvent = 0x22,
    TeleportEntity = 0x23,
    Explosion = 0x24,
    AddTransientBlock = 0x25,
    UnloadChunk = 0x26,
    GameEvent = 0x27,
    GameRuleValues = 0x28,
    GameTestHighlightPosition = 0x29,
    OpenHorseScreen = 0x2A,
    HurtAnimation = 0x2B,
    InitializeWorldBorder = 0x2C,
    KeepAliveClientbound = 0x2D,
    ChunkDataAndUpdateLight = 0x2E,
    WorldEvent = 0x2F,
    Particle = 0x30,
    UpdateLight = 0x31,
    LoginPlay = 0x32,
    LowDiskSpaceWarning = 0x33,
    MapData = 0x34,
    MerchantOffers = 0x35,
    UpdateEntityPosition = 0x36,
    UpdateEntityPositionAndRotation = 0x37,
    MoveMinecartAlongTrack = 0x38,
    UpdateEntityRotation = 0x39,
    MoveVehicleClientbound = 0x3A,
    OpenBook = 0x3B,
    OpenScreen = 0x3C,
    OpenSignEditor = 0x3D,
    Ping = 0x3E,
    PongResponse = 0x3F,
    PlaceGhostRecipe = 0x40,
    PlayerAbilitiesClientbound = 0x41,
    PlayerChatMessage = 0x42,
    EndCombat = 0x43,
    EnterCombat = 0x44,
    CombatDeath = 0x45,
    PlayerInfoRemove = 0x46,
    PlayerInfoUpdate = 0x47,
    LookAt = 0x48,
    SynchronizePlayerPosition = 0x49,
    PlayerRotation = 0x4A,
    RecipeBookAdd = 0x4B,
    RecipeBookRemove = 0x4C,
    RecipeBookSettings = 0x4D,
    RemoveEntities = 0x4E,
    RemoveEntityEffect = 0x4F,
    ResetScore = 0x50,
    RemoveResourcePack = 0x51,
    AddResourcePack = 0x52,
    PostEffects = 0x53,
    Respawn = 0x54,
    SetHeadRotation = 0x55,
    UpdateSectionBlocks = 0x56,
    SelectAdvancementsTab = 0x57,
    ServerData = 0x58,
    SetActionBarText = 0x59,
    SetBorderCenter = 0x5A,
    SetBorderLerpSize = 0x5B,
    SetBorderSize = 0x5C,
    SetBorderWarningDelay = 0x5D,
    SetBorderWarningDistance = 0x5E,
    SetCamera = 0x5F,
    SetCenterChunk = 0x60,
    SetRenderDistance = 0x61,
    SetCursorItem = 0x62,
    SetDefaultSpawnPosition = 0x63,
    DisplayObjective = 0x64,
    SetEntityMetadata = 0x65,
    LinkEntities = 0x66,
    SetEntityVelocity = 0x67,
    SetEquipment = 0x68,
    SetExperience = 0x69,
    SetHealth = 0x6A,
    SetHeldItemClientbound = 0x6B,
    UpdateObjectives = 0x6C,
    SetPassengers = 0x6D,
    SetPlayerInventorySlot = 0x6E,
    UpdateTeams = 0x6F,
    UpdateScore = 0x70,
    SetSimulationDistance = 0x71,
    SetSubtitleText = 0x72,
    UpdateTime = 0x73,
    SetTitleText = 0x74,
    SetTitleAnimationTimes = 0x75,
    EntitySoundEffect = 0x76,
    SoundEffect = 0x77,
    StartConfig = 0x78,
    StopSound = 0x79,
    StoreCookie = 0x7A,
    SwingAnimation = 0x7B,
    SystemChatMessage = 0x7C,
    SetTabListHeaderAndFooter = 0x7D,
    TagQueryResponse = 0x7E,
    PickupItem = 0x7F,
    SynchronizeVehiclePosition = 0x80,
    TestInstanceBlockStatus = 0x81,
    SetTickingStat = 0x82,
    StepTick = 0x83,
    Transfer = 0x84,
    UpdateAdvancements = 0x85,
    UpdateAttributes = 0x86,
    EntityEffect = 0x87,
    UpdateRecipes = 0x88,
    UpdateTags = 0x89,
    ProjectilePower = 0x8A,
    CustomServerDetails = 0x8B,
    ServerLinks = 0x8C,
    Waypoint = 0x8D,
    ClearDialog = 0x8E,
    ShowDialogPlay = 0x8F,
}
