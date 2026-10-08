-- ArpgHud: the benilla ARPG client's hover health bar, written into AddOns by the client while the
-- ARPG view is on (crates/benilla-app/src/player/arpg.rs). There is no target in the ARPG view, so
-- the enemy under the cursor ("mouseover") gets a bar at the top of the screen instead. The
-- player's health and power show as two orbs at the bottom corners.

local WIDTH, HEIGHT = 360, 18

local bar = CreateFrame("StatusBar", "ArpgHudBar", UIParent)
bar:SetWidth(WIDTH)
bar:SetHeight(HEIGHT)
bar:SetPoint("TOP", UIParent, "TOP", 0, -48)
bar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
bar:SetStatusBarColor(0.75, 0.08, 0.08)
bar:SetMinMaxValues(0, 1)
bar:SetValue(1)
bar:SetFrameStrata("HIGH")

local back = bar:CreateTexture(nil, "BACKGROUND")
back:SetAllPoints(bar)
back:SetTexture(0, 0, 0, 0.65)

local name = bar:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
name:SetPoint("CENTER", bar, "CENTER", 0, 0)

local rank = bar:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
rank:SetPoint("BOTTOM", bar, "TOP", 0, 3)

bar:Hide()

-- Gold for elites and bosses, silver for rares, as the stock target frame's dragon.
local RANKS = {
    worldboss = { "Boss", 1.0, 0.5, 0.0 },
    elite = { "Elite", 1.0, 0.82, 0.0 },
    rareelite = { "Rare Elite", 0.8, 0.8, 1.0 },
    rare = { "Rare", 0.8, 0.8, 1.0 },
}

local function refresh()
    if not UnitExists("mouseover") or not UnitCanAttack("player", "mouseover")
        or UnitIsDead("mouseover") then
        bar:Hide()
        return
    end
    local health, most = UnitHealth("mouseover"), UnitHealthMax("mouseover")
    if not most or most <= 0 then
        most = 1
    end
    bar:SetMinMaxValues(0, most)
    bar:SetValue(health or 0)
    name:SetText(UnitName("mouseover"))

    local level = UnitLevel("mouseover")
    local label = (level and level > 0) and ("Level " .. level) or "Level ??"
    local special = RANKS[UnitClassification("mouseover") or "normal"]
    if special then
        rank:SetText(label .. "  " .. special[1])
        rank:SetTextColor(special[2], special[3], special[4])
    else
        rank:SetText(label)
        rank:SetTextColor(1.0, 0.82, 0.0)
    end
    bar:Show()
end

-- A few refreshes a second follow the hover and the health without an event per unit.
local driver = CreateFrame("Frame")
local since = 0
driver:SetScript("OnUpdate", function()
    since = since + (arg1 or 0)
    if since >= 0.05 then
        since = 0
        refresh()
    end
end)
driver:RegisterEvent("UPDATE_MOUSEOVER_UNIT")
driver:SetScript("OnEvent", refresh)

-- The health and power orbs, Diablo's globes, at the bottom corners of the screen. Each is a
-- stack of horizontal slices cut to a circle; the slices under the fill line take the orb's colour,
-- shaded darker toward the rim, and those above it stay dark glass. The power orb takes the
-- player's power colour: blue mana, red rage, yellow energy.

local ORB_RADIUS, ORB_SLICES, ORB_INSET = 56, 48, 18
local RIM = 3

local POWER_COLORS = {
    [0] = { 0.12, 0.30, 0.95 }, -- mana
    [1] = { 0.85, 0.10, 0.08 }, -- rage
    [2] = { 0.95, 0.55, 0.15 }, -- focus
    [3] = { 0.95, 0.85, 0.15 }, -- energy
}
local HEALTH_COLOR = { 0.80, 0.06, 0.06 }

local function makeOrb(frameName, corner, x)
    local size = ORB_RADIUS * 2
    local orb = CreateFrame("Frame", frameName, UIParent)
    orb:SetWidth(size + RIM * 2)
    orb:SetHeight(size + RIM * 2)
    orb:SetPoint(corner, UIParent, corner, x, ORB_INSET)
    orb:SetFrameStrata("MEDIUM")

    local step = size / ORB_SLICES
    orb.slices = {}
    orb.shade = {}
    for i = 1, ORB_SLICES do
        -- The slice's centre height over the orb's centre, and its half-width on the circle.
        local y = (i - 0.5) * step - ORB_RADIUS
        local half = math.sqrt(math.max(ORB_RADIUS * ORB_RADIUS - y * y, 0))
        -- The rim: a dark band either side of the glass.
        local rhalf = math.sqrt(math.max((ORB_RADIUS + RIM) * (ORB_RADIUS + RIM) - y * y, 0))
        local rim = orb:CreateTexture(nil, "BACKGROUND")
        rim:SetTexture(0.05, 0.04, 0.03, 0.9)
        rim:SetWidth(math.max(rhalf * 2, 1))
        rim:SetHeight(step + 0.5)
        rim:SetPoint("BOTTOM", orb, "BOTTOM", 0, RIM + (i - 1) * step)

        local slice = orb:CreateTexture(nil, "ARTWORK")
        slice:SetWidth(math.max(half * 2, 1))
        slice:SetHeight(step + 0.5)
        slice:SetPoint("BOTTOM", orb, "BOTTOM", 0, RIM + (i - 1) * step)
        orb.slices[i] = slice
        -- Lighter in the upper middle, darker at the bottom: a little roundness.
        orb.shade[i] = 0.55 + 0.45 * math.sqrt(math.sin(math.pi * (i - 0.5) / ORB_SLICES))
            * (0.75 + 0.25 * (i / ORB_SLICES))
    end
    -- The glare: a pale streak on the glass's upper left.
    local glare = orb:CreateTexture(nil, "OVERLAY")
    glare:SetTexture(1, 1, 1, 0.12)
    glare:SetWidth(ORB_RADIUS * 0.5)
    glare:SetHeight(ORB_RADIUS * 0.22)
    glare:SetPoint("CENTER", orb, "CENTER", -ORB_RADIUS * 0.3, ORB_RADIUS * 0.55)

    orb.text = orb:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    orb.text:SetPoint("CENTER", orb, "CENTER", 0, 0)
    orb.filled = -1
    orb.r, orb.g, orb.b = -1, -1, -1
    return orb
end

local healthOrb = makeOrb("ArpgHudHealthOrb", "BOTTOMLEFT", ORB_INSET)
local powerOrb = makeOrb("ArpgHudPowerOrb", "BOTTOMRIGHT", -ORB_INSET)

-- Paint the orb `fraction` full in this colour; only a change repaints the slices.
local function paintOrb(orb, fraction, color, value)
    local filled = math.floor(math.max(math.min(fraction, 1), 0) * ORB_SLICES + 0.5)
    local r, g, b = color[1], color[2], color[3]
    if filled ~= orb.filled or r ~= orb.r or g ~= orb.g or b ~= orb.b then
        for i = 1, ORB_SLICES do
            local k = orb.shade[i]
            if i <= filled then
                orb.slices[i]:SetTexture(r * k, g * k, b * k, 0.95)
            else
                orb.slices[i]:SetTexture(0.06 * k, 0.05 * k, 0.07 * k, 0.85)
            end
        end
        orb.filled, orb.r, orb.g, orb.b = filled, r, g, b
    end
    orb.text:SetText(value)
end

local function refreshOrbs()
    local health, most = UnitHealth("player") or 0, UnitHealthMax("player") or 0
    paintOrb(healthOrb, most > 0 and health / most or 0, HEALTH_COLOR, health)
    local power, top = UnitMana("player") or 0, UnitManaMax("player") or 0
    local color = POWER_COLORS[UnitPowerType("player") or 0] or POWER_COLORS[0]
    paintOrb(powerOrb, top > 0 and power / top or 0, color, power)
end

local orbDriver = CreateFrame("Frame")
local orbSince = 0
orbDriver:SetScript("OnUpdate", function()
    orbSince = orbSince + (arg1 or 0)
    if orbSince >= 0.05 then
        orbSince = 0
        refreshOrbs()
    end
end)
