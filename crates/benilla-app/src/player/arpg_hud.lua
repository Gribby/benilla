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

-- A champion's or a rare's affixes, under the bar.
local affix = bar:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
affix:SetPoint("TOP", bar, "BOTTOM", 0, -3)
affix:SetTextColor(0.85, 0.85, 0.85)

bar:Hide()

-- The champion or rare under the cursor, which the client sets as the hover moves:
-- ArpgHud_Champion(tier, name, affixes), tier 1 a champion, 2 a rare; ArpgHud_Champion(0) clears.
local champ = nil
function ArpgHud_Champion(tier, champName, affixes)
    if not tier or tier <= 0 then
        champ = nil
    else
        champ = { tier = tier, name = champName or "", affixes = affixes or "" }
    end
end

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
    local level = UnitLevel("mouseover")
    local label = (level and level > 0) and ("Level " .. level) or "Level ??"
    local special = RANKS[UnitClassification("mouseover") or "normal"]
    if champ then
        -- Champions blue, rares yellow, as their rings.
        local r, g, b, what = 0.30, 0.55, 1.0, "Champion"
        if champ.tier >= 2 then
            r, g, b, what = 1.0, 0.85, 0.25, "Rare"
        end
        local shown = champ.name
        if shown == "" then shown = UnitName("mouseover") end
        name:SetText(shown)
        name:SetTextColor(r, g, b)
        rank:SetText(label .. "  " .. what)
        rank:SetTextColor(r, g, b)
        affix:SetText(champ.affixes)
        affix:Show()
        bar:Show()
        return
    end
    name:SetText(UnitName("mouseover"))
    name:SetTextColor(1.0, 1.0, 1.0)
    affix:Hide()
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

-- The health and power orbs, Diablo's globes, at the bottom corners of the screen. An orb is a
-- glass circle in a bronze rim, cut into thin vertical columns; each column's liquid rises to the
-- fill line, and the line sloshes: two slow waves run across its surface, it eases to a new level instead of
-- jumping, and a few bubbles rise through it. The surface wears a bright meniscus. The power orb
-- takes the player's power colour: blue mana, red rage, yellow energy.

local ORB_RADIUS, ORB_COLS, ORB_INSET = 56, 44, 18
local RIM = 4
local ORB_BUBBLES = 6

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

    local step = size / ORB_COLS
    orb.cols = {}
    for i = 1, ORB_COLS do
        -- The column's centre across the orb, and the circle's half-height there.
        local cx = (i - 0.5) * step - ORB_RADIUS
        local half = math.sqrt(math.max(ORB_RADIUS * ORB_RADIUS - cx * cx, 0))
        local rhalf = math.sqrt(math.max((ORB_RADIUS + RIM) * (ORB_RADIUS + RIM) - cx * cx, 0))
        local rim = orb:CreateTexture(nil, "BACKGROUND")
        -- Bronze, lit from above.
        rim:SetTexture(0.36, 0.27, 0.12, 1)
        rim:SetWidth(step + 0.5)
        rim:SetHeight(math.max(rhalf * 2, 1))
        rim:SetPoint("CENTER", orb, "CENTER", cx, 0)
        local glass = orb:CreateTexture(nil, "BORDER")
        local k = 0.6 + 0.4 * math.cos(cx / ORB_RADIUS * 1.4)
        glass:SetTexture(0.05 * k, 0.045 * k, 0.06 * k, 0.92)
        glass:SetWidth(step + 0.5)
        glass:SetHeight(math.max(half * 2, 1))
        glass:SetPoint("CENTER", orb, "CENTER", cx, 0)
        local liquid = orb:CreateTexture(nil, "ARTWORK")
        liquid:SetWidth(step + 0.5)
        liquid:SetPoint("BOTTOM", orb, "CENTER", cx, -half)
        local foam = orb:CreateTexture(nil, "OVERLAY")
        foam:SetWidth(step + 0.5)
        foam:SetHeight(1.5)
        liquid:Hide()
        foam:Hide()
        orb.cols[i] = { x = cx, half = half, liquid = liquid, foam = foam, wet = false, foamy = false,
            -- Lighter left of centre, darker at the rim: a lit sphere.
            shade = 0.45 + 0.55 * math.cos((cx + ORB_RADIUS * 0.25) / ORB_RADIUS * 1.25) }
    end
    orb.bubbles = {}
    for i = 1, ORB_BUBBLES do
        local b = orb:CreateTexture(nil, "OVERLAY")
        b:SetWidth(2.5)
        b:SetHeight(2.5)
        b:Hide()
        orb.bubbles[i] = { tex = b, x = 0, y = -ORB_RADIUS, speed = 0, live = false }
    end
    -- The glare: two pale streaks on the glass's upper left.
    local glare = orb:CreateTexture(nil, "OVERLAY")
    glare:SetTexture(1, 1, 1, 0.14)
    glare:SetWidth(ORB_RADIUS * 0.5)
    glare:SetHeight(ORB_RADIUS * 0.16)
    glare:SetPoint("CENTER", orb, "CENTER", -ORB_RADIUS * 0.32, ORB_RADIUS * 0.58)
    local glint = orb:CreateTexture(nil, "OVERLAY")
    glint:SetTexture(1, 1, 1, 0.22)
    glint:SetWidth(ORB_RADIUS * 0.12)
    glint:SetHeight(ORB_RADIUS * 0.12)
    glint:SetPoint("CENTER", orb, "CENTER", -ORB_RADIUS * 0.55, ORB_RADIUS * 0.3)
    -- The number, over the liquid and the glare.
    local face = CreateFrame("Frame", nil, orb)
    face:SetFrameLevel(orb:GetFrameLevel() + 2)
    face:SetAllPoints(orb)
    orb.text = face:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    orb.text:SetPoint("CENTER", orb, "CENTER", 0, 0)
    orb.shown, orb.target = 0, 0
    orb.r, orb.g, orb.b = 0.5, 0.5, 0.5
    orb.phase = math.random() * 6.28
    return orb
end

local healthOrb = makeOrb("ArpgHudHealthOrb", "BOTTOMLEFT", ORB_INSET)
local powerOrb = makeOrb("ArpgHudPowerOrb", "BOTTOMRIGHT", -ORB_INSET)

-- Fork-only: the combo points over the power orb (ArpgHud_Combo, pushed from actions.rs). An ARPG
-- character's points carry from enemy to enemy, so the count shows whatever is targeted; with
-- none, the pips hide.
local COMBO_PIP = 9
local COMBO_LIT = { 1.0, 0.82, 0.2 }
local COMBO_DARK = { 0.25, 0.18, 0.05 }
local comboPips = {}
for i = 1, 5 do
    local pip = powerOrb:CreateTexture(nil, "OVERLAY")
    pip:SetWidth(COMBO_PIP)
    pip:SetHeight(COMBO_PIP)
    pip:SetPoint("BOTTOM", powerOrb, "TOP", (i - 3) * (COMBO_PIP + 4), 4)
    pip:Hide()
    comboPips[i] = pip
end

function ArpgHud_Combo(points)
    for i = 1, 5 do
        local pip = comboPips[i]
        if points > 0 then
            local c = COMBO_DARK
            if i <= points then c = COMBO_LIT end
            pip:SetTexture(c[1], c[2], c[3], 1)
            pip:Show()
        else
            pip:Hide()
        end
    end
end

-- The orb is to stand `fraction` full in this colour; animateOrb carries it there.
local function paintOrb(orb, fraction, color, value)
    orb.target = math.max(math.min(fraction, 1), 0)
    orb.r, orb.g, orb.b = color[1], color[2], color[3]
    orb.text:SetText(value)
end

-- One frame of an orb: the level eases toward its target, the surface waves, bubbles rise.
local function animateOrb(orb, dt, t)
    orb.shown = orb.shown + (orb.target - orb.shown) * math.min(1, dt * 6)
    if math.abs(orb.target - orb.shown) < 0.001 then orb.shown = orb.target end
    local level = -ORB_RADIUS + 2 * ORB_RADIUS * orb.shown
    -- No waves on an empty or a brimming orb.
    local calm = math.min(orb.shown, 1 - orb.shown) * 8
    local amp = 2.2 * math.max(0, math.min(1, calm))
    local r, g, b = orb.r, orb.g, orb.b
    -- The colours only when they change (a shapeshift, a new power type).
    local recolour = r ~= orb.pr or g ~= orb.pg or b ~= orb.pb
    orb.pr, orb.pg, orb.pb = r, g, b
    for i = 1, ORB_COLS do
        local c = orb.cols[i]
        if recolour then
            local k = c.shade
            c.liquid:SetTexture(r * k, g * k, b * k, 0.96)
            c.foam:SetTexture(math.min(1, r * 0.6 + 0.45), math.min(1, g * 0.6 + 0.45), math.min(1, b * 0.6 + 0.45), 0.8)
        end
        local wave = amp * (math.sin(c.x * 0.11 + t * 2.1 + orb.phase) + 0.5 * math.sin(c.x * 0.29 - t * 3.3))
        local top = math.min(level + wave, c.half)
        local height = top + c.half
        local wet = height > 0.5
        local foamy = wet and top < c.half - 1
        if wet ~= c.wet then
            if wet then c.liquid:Show() else c.liquid:Hide() end
            c.wet = wet
        end
        local moved = wet and math.abs(height - (c.height or -1)) > 0.25
        if foamy ~= c.foamy then
            if foamy then c.foam:Show() else c.foam:Hide() end
            c.foamy = foamy
            moved = moved or foamy
        end
        if moved then
            c.liquid:SetHeight(height)
            c.height = height
            if foamy then
                c.foam:ClearAllPoints()
                c.foam:SetPoint("TOP", orb, "CENTER", c.x, top)
            end
        end
    end
    for i = 1, ORB_BUBBLES do
        local bub = orb.bubbles[i]
        if not bub.live then
            if orb.shown > 0.08 and math.random() < dt * 0.9 then
                bub.live = true
                bub.x = (math.random() - 0.5) * ORB_RADIUS * 1.2
                bub.y = -ORB_RADIUS * 0.85
                bub.speed = 10 + math.random() * 14
            end
        else
            bub.y = bub.y + bub.speed * dt
            local wobble = math.sin(t * 6 + i) * 1.2
            if bub.y >= level - 2 then
                bub.live = false
                bub.tex:Hide()
            else
                bub.tex:SetTexture(math.min(1, r + 0.4), math.min(1, g + 0.4), math.min(1, b + 0.4), 0.55)
                bub.tex:ClearAllPoints()
                bub.tex:SetPoint("CENTER", orb, "CENTER", bub.x + wobble, bub.y)
                bub.tex:Show()
            end
        end
    end
end

local function refreshOrbs()
    local health, most = UnitHealth("player") or 0, UnitHealthMax("player") or 0
    paintOrb(healthOrb, most > 0 and health / most or 0, HEALTH_COLOR, health)
    local power, top = UnitMana("player") or 0, UnitManaMax("player") or 0
    local color = POWER_COLORS[UnitPowerType("player") or 0] or POWER_COLORS[0]
    paintOrb(powerOrb, top > 0 and power / top or 0, color, power)
end

-- The flask (Q) and the dodge roll (Space): the server's charges and cooldown, which the client
-- passes on through ArpgHud_Status. The flask's charges stand as vials left of the skill bar, the
-- next one filling as kills come in; the roll's bar right of it fills back up after a roll.
local VIAL_W, VIAL_H, VIAL_GAP = 12, 34, 5
-- Where the skill bar's ends are (the bottom's layout, below): half its width, and its foot.
local BAR_HALF, BAR_FOOT = (12 * 36 + 11 * 6) / 2 + 8, 8
local vials = {}
local function makeVial(i)
    local v = CreateFrame("Frame", nil, UIParent)
    v:SetWidth(VIAL_W)
    v:SetHeight(VIAL_H)
    -- Right to left from the skill bar's left end, so they grow toward the health orb.
    v:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOM", -(BAR_HALF + 14 + (i - 1) * (VIAL_W + VIAL_GAP)), BAR_FOOT)
    v:SetFrameStrata("MEDIUM")
    local back = v:CreateTexture(nil, "BACKGROUND")
    back:SetAllPoints(v)
    back:SetTexture(0.05, 0.04, 0.03, 0.85)
    v.fill = v:CreateTexture(nil, "ARTWORK")
    v.fill:SetPoint("BOTTOMLEFT", v, "BOTTOMLEFT", 2, 2)
    v.fill:SetWidth(VIAL_W - 4)
    v.fill:SetHeight(1)
    v:Hide()
    return v
end

local flaskLabel = UIParent:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
flaskLabel:SetPoint("BOTTOMRIGHT", UIParent, "BOTTOM", -(BAR_HALF + 14), BAR_FOOT + VIAL_H + 4)
flaskLabel:SetText("Q")
flaskLabel:Hide()

local dodgeBar = CreateFrame("StatusBar", nil, UIParent)
dodgeBar:SetWidth(70)
dodgeBar:SetHeight(8)
dodgeBar:SetPoint("BOTTOMLEFT", UIParent, "BOTTOM", BAR_HALF + 14, BAR_FOOT + 6)
dodgeBar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
dodgeBar:SetStatusBarColor(0.85, 0.8, 0.55)
dodgeBar:SetMinMaxValues(0, 1)
dodgeBar:SetValue(1)
dodgeBar:SetFrameStrata("MEDIUM")
local dodgeBack = dodgeBar:CreateTexture(nil, "BACKGROUND")
dodgeBack:SetAllPoints(dodgeBar)
dodgeBack:SetTexture(0, 0, 0, 0.7)
local dodgeLabel = dodgeBar:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
dodgeLabel:SetPoint("BOTTOM", dodgeBar, "TOP", 0, 2)
dodgeLabel:SetText("Space: Roll")
dodgeBar:Hide()

local flask = { charges = 0, max = 0, progress = 0 }
local dodgeReadyAt, dodgeCooldown = 0, 2.5

function ArpgHud_Status(charges, max, progress, dodgeMs, cooldownMs)
    flask.charges, flask.max, flask.progress = charges or 0, max or 0, progress or 0
    dodgeReadyAt = GetTime() + (dodgeMs or 0) / 1000
    if cooldownMs and cooldownMs > 0 then
        dodgeCooldown = cooldownMs / 1000
    end
    for i = 1, flask.max do
        local v = vials[i] or makeVial(i)
        vials[i] = v
        local h = VIAL_H - 4
        if i <= flask.charges then
            v.fill:SetHeight(h)
            v.fill:SetTexture(0.85, 0.12, 0.12, 0.95)
        elseif i == flask.charges + 1 and flask.progress > 0 then
            v.fill:SetHeight(math.max(1, h * flask.progress / 100))
            v.fill:SetTexture(0.45, 0.10, 0.10, 0.9)
        else
            v.fill:SetHeight(1)
            v.fill:SetTexture(0, 0, 0, 0)
        end
        v:Show()
    end
    flaskLabel:Show()
    dodgeBar:Show()
end

local function refreshDodge()
    if not dodgeBar:IsShown() then
        return
    end
    local left = dodgeReadyAt - GetTime()
    if left <= 0 then
        dodgeBar:SetValue(1)
        dodgeBar:SetStatusBarColor(0.85, 0.8, 0.55)
    else
        dodgeBar:SetValue(1 - left / dodgeCooldown)
        dodgeBar:SetStatusBarColor(0.45, 0.42, 0.3)
    end
end

local orbDriver = CreateFrame("Frame")
local orbSince = 0
local orbClock = 0
orbDriver:SetScript("OnUpdate", function()
    local dt = math.min(arg1 or 0, 0.1)
    orbClock = orbClock + dt
    if healthOrb:IsVisible() then animateOrb(healthOrb, dt, orbClock) end
    if powerOrb:IsVisible() then animateOrb(powerOrb, dt, orbClock) end
    orbSince = orbSince + (arg1 or 0)
    if orbSince >= 0.05 then
        orbSince = 0
        refreshOrbs()
        refreshDodge()
    end
end)

-- ── Feel: the screen's edges and the kill streak ──
-- The edges glow red as health runs low, beating like a heart below a third, flash red when a
-- big blow lands (a tenth of the health at once), and flash in a champion's or a rare's colour
-- at its kill (ArpgHud_Flash). The UI draws a texture in one colour, so the fade from the edge is
-- built of bands, each fainter than the one outside it.
local EDGE_BANDS, EDGE_DEPTH = 8, 120
local LOW_HEALTH, LOW_ALPHA = 0.35, 0.5
local BEAT_SECS = 1.1
local edges = CreateFrame("Frame", "ArpgHudEdges", UIParent)
edges:SetAllPoints(UIParent)
edges:SetFrameStrata("BACKGROUND")
edges:EnableMouse(false)
edges.tex = {}
for i = 1, EDGE_BANDS do
    local inset = (i - 1) * EDGE_DEPTH / EDGE_BANDS
    local thick = EDGE_DEPTH / EDGE_BANDS
    local fade = 1 - (i - 1) / EDGE_BANDS
    local weight = fade * fade * 0.35
    local top = edges:CreateTexture(nil, "BACKGROUND")
    top:SetPoint("TOPLEFT", edges, "TOPLEFT", inset, -inset)
    top:SetPoint("TOPRIGHT", edges, "TOPRIGHT", -inset, -inset)
    top:SetHeight(thick)
    local bottom = edges:CreateTexture(nil, "BACKGROUND")
    bottom:SetPoint("BOTTOMLEFT", edges, "BOTTOMLEFT", inset, inset)
    bottom:SetPoint("BOTTOMRIGHT", edges, "BOTTOMRIGHT", -inset, inset)
    bottom:SetHeight(thick)
    local left = edges:CreateTexture(nil, "BACKGROUND")
    left:SetPoint("TOPLEFT", edges, "TOPLEFT", inset, -(inset + thick))
    left:SetPoint("BOTTOMLEFT", edges, "BOTTOMLEFT", inset, inset + thick)
    left:SetWidth(thick)
    local right = edges:CreateTexture(nil, "BACKGROUND")
    right:SetPoint("TOPRIGHT", edges, "TOPRIGHT", -inset, -(inset + thick))
    right:SetPoint("BOTTOMRIGHT", edges, "BOTTOMRIGHT", -inset, inset + thick)
    right:SetWidth(thick)
    table.insert(edges.tex, { top, weight })
    table.insert(edges.tex, { bottom, weight })
    table.insert(edges.tex, { left, weight })
    table.insert(edges.tex, { right, weight })
end
edges.flashA, edges.flashR, edges.flashG, edges.flashB = 0, 1, 0, 0
edges.shown = -1
edges.lastHealth = nil
edges:Hide()

local function edgesPaint(r, g, b, a)
    if a < 0.01 then
        if edges:IsShown() then edges:Hide() end
        edges.shown = 0
        return
    end
    if math.abs(a - edges.shown) < 0.01 and r == edges.pr and g == edges.pg and b == edges.pb then return end
    edges.shown, edges.pr, edges.pg, edges.pb = a, r, g, b
    for i = 1, table.getn(edges.tex) do
        local t = edges.tex[i]
        t[1]:SetTexture(r, g, b, math.min(1, a * t[2] / 0.35))
    end
    if not edges:IsShown() then edges:Show() end
end

-- A heartbeat: two quick bumps each beat, `t` in seconds.
local function heartbeat(t)
    local phase = math.mod(t, BEAT_SECS) / BEAT_SECS
    local function bump(at, width)
        local d = (phase - at) / width
        if d < -1 or d > 1 then return 0 end
        return 1 - d * d
    end
    return math.max(bump(0.1, 0.08), 0.6 * bump(0.28, 0.08))
end

function ArpgHud_Flash(r, g, b)
    edges.flashA, edges.flashR, edges.flashG, edges.flashB = 0.9, r, g, b
end

local edgeClock = 0
local function refreshEdges(dt)
    edgeClock = edgeClock + dt
    local health, most = UnitHealth("player") or 0, UnitHealthMax("player") or 0
    local dead = UnitIsDeadOrGhost("player")
    -- A big blow: a tenth of the health gone since the last look.
    if edges.lastHealth and most > 0 and not dead and edges.lastHealth - health >= most * 0.1 then
        if edges.flashA < 0.6 then
            edges.flashA, edges.flashR, edges.flashG, edges.flashB = 0.6, 0.9, 0.05, 0.05
        end
    end
    edges.lastHealth = health
    edges.flashA = math.max(0, edges.flashA - dt * 2.4)
    local low = 0
    if most > 0 and not dead then
        local f = health / most
        if f < LOW_HEALTH then
            low = (LOW_HEALTH - f) / LOW_HEALTH * LOW_ALPHA
            if f < LOW_HEALTH * 0.6 then
                low = low * (0.55 + 0.45 * heartbeat(edgeClock))
            end
        end
    end
    if edges.flashA > low then
        edgesPaint(edges.flashR, edges.flashG, edges.flashB, edges.flashA)
    else
        edgesPaint(0.75, 0.02, 0.02, low)
    end
end

-- The kill streak: kills in a row (each within a few seconds of the last), from three up, above
-- the skill bar. Each kill pops it; it fades when the streak ends (STREAK_SECS, the client's
-- STREAK_WINDOW_SECS); 10, 25, 50 and 100 name the streak and sound.
local STREAK_SECS = 4.0
local streak = CreateFrame("Frame", "ArpgHudStreak", UIParent)
streak:SetWidth(320)
streak:SetHeight(70)
streak:SetPoint("BOTTOM", UIParent, "BOTTOM", 0, 120)
streak:SetFrameStrata("MEDIUM")
streak:EnableMouse(false)
-- The pop scales an inner frame centred on the streak's, so the anchor's offset stays put.
streak.face = CreateFrame("Frame", nil, streak)
streak.face:SetWidth(320)
streak.face:SetHeight(70)
streak.face:SetPoint("CENTER", streak, "CENTER", 0, 0)
-- A font string in font object `font`, or a plain outlined face of `size` without it.
local function streakText(font, size)
    local fs = streak.face:CreateFontString(nil, "OVERLAY")
    if font then
        fs:SetFontObject(font)
    else
        fs:SetFont("Fonts\\FRIZQT__.TTF", size, "OUTLINE")
    end
    return fs
end
streak.count = streakText(GameFontNormalHuge, 26)
streak.count:SetPoint("CENTER", streak.face, "CENTER", 0, 10)
streak.word = streakText(GameFontNormalLarge, 16)
streak.word:SetPoint("TOP", streak.count, "BOTTOM", 0, -2)
streak.left, streak.pop, streak.wordLeft = 0, 0, 0
streak:Hide()

local STREAK_COLORS = {
    { 3, 1.0, 0.95, 0.8 },
    { 10, 1.0, 0.6, 0.15 },
    { 25, 1.0, 0.25, 0.15 },
    { 50, 0.8, 0.35, 1.0 },
    { 100, 1.0, 0.82, 0.2 },
}

function ArpgHud_Streak(n, word)
    if n < 3 then
        streak.left = 0
        return
    end
    local r, g, b = 1, 1, 1
    for i = 1, table.getn(STREAK_COLORS) do
        local c = STREAK_COLORS[i]
        if n >= c[1] then r, g, b = c[2], c[3], c[4] end
    end
    streak.count:SetText(n .. " Kills")
    streak.count:SetTextColor(r, g, b)
    streak.left, streak.pop = STREAK_SECS, 1
    if word then
        streak.word:SetText(word)
        streak.word:SetTextColor(r, g, b)
        streak.wordLeft = 3.0
        if n >= 50 then PlaySound("LEVELUP") else PlaySound("igQuestListComplete") end
    end
    streak:Show()
end

local function refreshStreak(dt)
    if not streak:IsShown() then return end
    streak.left = streak.left - dt
    streak.wordLeft = streak.wordLeft - dt
    streak.pop = math.max(0, streak.pop - dt * 6)
    if streak.left <= 0 then
        streak:Hide()
        return
    end
    streak.face:SetScale(1 + 0.35 * streak.pop * streak.pop)
    streak:SetAlpha(math.min(1, streak.left / 0.8))
    if streak.wordLeft > 0 then streak.word:Show() else streak.word:Hide() end
end

local feelDriver = CreateFrame("Frame")
feelDriver:SetScript("OnUpdate", function()
    local dt = math.min(arg1 or 0, 0.1)
    refreshEdges(dt)
    refreshStreak(dt)
end)

-- ── The bottom of the screen ──
-- The stock main bar is a 1024-pixel strip of art, gryphons, paging arrows, micro-menu and bag
-- buttons, which covers the orbs' neighbours and takes the world's clicks along the bottom. The
-- ARPG view keeps only what it plays with: the twelve action buttons in a compact row centred
-- between the orbs on a dark plate, a slim XP line above them, the flask's vials (Q) on its left
-- and the roll (Space) on its right. The rest is parked in a hidden frame; its keys (C, P, B, M,
-- Esc and the rest) still open their windows. The stance, pet and cast bars sit above the row.

local BAR_BUTTON, BAR_GAP, BAR_BOTTOM = 36, 6, 14
local BAR_WIDTH = 12 * BAR_BUTTON + 11 * BAR_GAP
local XP_HEIGHT = 5

local parked = CreateFrame("Frame", "ArpgHudParked", UIParent)
parked:Hide()

-- Piece by piece: the action buttons are the art frame's children, so it stays (empty).
local PARK = {
    "MainMenuBarLeftEndCap", "MainMenuBarRightEndCap",
    "MainMenuBarTexture0", "MainMenuBarTexture1", "MainMenuBarTexture2", "MainMenuBarTexture3",
    "MainMenuBarPageNumber", "ActionBarUpButton", "ActionBarDownButton",
    "MainMenuBarBackpackButton", "CharacterBag0Slot", "CharacterBag1Slot", "CharacterBag2Slot",
    "CharacterBag3Slot", "KeyRingButton",
    "CharacterMicroButton", "SpellbookMicroButton", "TalentMicroButton", "QuestLogMicroButton",
    "SocialsMicroButton", "WorldMapMicroButton", "MainMenuMicroButton", "HelpMicroButton",
    "MainMenuBarPerformanceBarFrame", "MainMenuBarMaxLevelBar", "ReputationWatchBar",
    "MainMenuXPBarTexture0", "MainMenuXPBarTexture1", "MainMenuXPBarTexture2",
    "MainMenuXPBarTexture3", "BonusActionBarTexture0", "BonusActionBarTexture1",
}

local plate = CreateFrame("Frame", "ArpgHudBar", UIParent)
plate:SetWidth(BAR_WIDTH + 16)
plate:SetHeight(BAR_BUTTON + XP_HEIGHT + 20)
plate:SetPoint("BOTTOM", UIParent, "BOTTOM", 0, BAR_BOTTOM - 8)
plate:SetFrameStrata("BACKGROUND")
local plateBack = plate:CreateTexture(nil, "BACKGROUND")
plateBack:SetAllPoints(plate)
plateBack:SetTexture(0.04, 0.035, 0.03, 0.78)
local plateEdge = plate:CreateTexture(nil, "BORDER")
plateEdge:SetPoint("TOPLEFT", plate, "TOPLEFT", 0, 0)
plateEdge:SetPoint("TOPRIGHT", plate, "TOPRIGHT", 0, 0)
plateEdge:SetHeight(1)
plateEdge:SetTexture(0.55, 0.45, 0.25, 0.8)

local function park(name)
    local f = getglobal(name)
    if not f then
        return
    end
    if f.SetParent then
        f:SetParent(parked)
    end
    f:Hide()
end

-- Lay the bottom out again: at load, and after every stock re-layout.
function ArpgHud_LayoutBar()
    if not MainMenuBar or not ActionButton1 then
        return
    end
    for _, name in ipairs(PARK) do
        park(name)
    end
    -- The empty strip must not eat the world's clicks.
    MainMenuBar:EnableMouse(false)
    if MainMenuBarArtFrame then
        MainMenuBarArtFrame:EnableMouse(false)
    end
    ActionButton1:ClearAllPoints()
    ActionButton1:SetPoint("BOTTOMLEFT", UIParent, "BOTTOM", -BAR_WIDTH / 2, BAR_BOTTOM)
    for i = 2, 12 do
        local b = getglobal("ActionButton" .. i)
        if b then
            b:ClearAllPoints()
            b:SetPoint("LEFT", getglobal("ActionButton" .. (i - 1)), "RIGHT", BAR_GAP, 0)
        end
    end
    -- A stance's or a form's bar (warrior, druid, stealth) lies over the same twelve slots.
    for i = 1, 12 do
        local b = getglobal("BonusActionButton" .. i)
        local under = getglobal("ActionButton" .. i)
        if b and under then
            b:ClearAllPoints()
            b:SetPoint("CENTER", under, "CENTER", 0, 0)
        end
    end
    if MainMenuExpBar then
        MainMenuExpBar:ClearAllPoints()
        MainMenuExpBar:SetPoint("BOTTOMLEFT", ActionButton1, "TOPLEFT", 0, 6)
        MainMenuExpBar:SetWidth(BAR_WIDTH)
        MainMenuExpBar:SetHeight(XP_HEIGHT)
    end
    local above = MainMenuExpBar or ActionButton1
    if ShapeshiftBarFrame then
        ShapeshiftBarFrame:ClearAllPoints()
        ShapeshiftBarFrame:SetPoint("BOTTOMLEFT", above, "TOPLEFT", 0, 8)
    end
    -- The pet bar slides in by itself from its parent's left edge (PetActionBar_OnUpdate): only
    -- its offset moves, over the left of the row (a pet class has no stance bar).
    if PETACTIONBAR_XPOS and MainMenuBar.GetWidth then
        PETACTIONBAR_XPOS = math.floor((MainMenuBar:GetWidth() - BAR_WIDTH) / 2 + 0.5)
    end
    -- Bags open above the orbs, not over the power orb.
    if CONTAINER_OFFSET_Y then
        CONTAINER_OFFSET_Y = math.max(CONTAINER_OFFSET_Y, 150)
    end
    if CastingBarFrame then
        CastingBarFrame:ClearAllPoints()
        CastingBarFrame:SetPoint("BOTTOM", UIParent, "BOTTOM", 0, BAR_BOTTOM + BAR_BUTTON + XP_HEIGHT + 64)
    end
end

-- The stock re-layout moves the stance, pet and cast bars back over the old strip: follow it.
if UIParent_ManageFramePositions then
    local stockManage = UIParent_ManageFramePositions
    UIParent_ManageFramePositions = function(a1, a2, a3)
        stockManage(a1, a2, a3)
        ArpgHud_LayoutBar()
    end
end
ArpgHud_LayoutBar()

-- ── The options window's ARPG page ──
-- benilla's options window (BenillaOptionsFrame) gains an "ARPG View" category under its own
-- header, built from the window's own row templates, so its rows store, refresh, search and reset
-- like every other page. Each slider writes its setting at once, and the client reads the settings
-- every frame, so the camera and the cut move as the thumb does. A bare VM has no window: skip.

ARPG_TOOLTIP_CAMERA_YAW = "Turns the camera around your character, in degrees."
ARPG_TOOLTIP_CAMERA_PITCH = "How steeply the camera looks down, in degrees: 90 is straight down."
ARPG_TOOLTIP_CAMERA_DISTANCE = "How far the camera sits from your character, in yards."
ARPG_TOOLTIP_CUT_HEIGHT = "Indoors and in caves, walls and ceilings more than this many yards "
    .. "above your feet are cut away. Lower shows more of the room; higher keeps more walls."
ARPG_TOOLTIP_CUT_RADIUS = "Indoors and in caves, how far from your character the cut reaches, "
    .. "in yards. Geometry farther out keeps its tops."
ARPG_TOOLTIP_LOOT_FILTER = "Which items on the ground get a name label. Hidden ones still "
    .. "glow and can be picked up, and holding Alt shows every label. Gold always shows."
ARPG_TOOLTIP_DEV_LOOT = "For testing: Drop Test Loot asks the server for a corpse at your feet "
    .. "holding this many items of this quality around this item level (past 60, the raids'). "
    .. "Quality is the rolled rarity: Purple is an ordinary base item rolled with 5-6 affixes, "
    .. "Unique a named item. The Dungeon Tier sets how strong the affixes roll. Form Test Pack makes the "
    .. "nearest mob lead a pack of five; Champion Pack and Rare Pack bring champions or a rare. "
    .. "All need Arpg.DevTools = 1 in the server's mangosd.conf."
ARPG_TOOLTIP_DUNGEON_TIER = "The difficulty of the next dungeon you enter first: tougher, harder-"
    .. "hitting monsters with more affixes, for more XP and better loot. Clearing a dungeon's final "
    .. "boss opens its next tier. Q drinks your flask; Space rolls."
ARPG_TOOLTIP_SEE_THROUGH = "Outdoors, roofs, awnings and trees between your character and the "
    .. "camera turn see-through."

-- `cvar`, the label, the tooltip global, then a slider's min, max, step and readout format (or,
-- with no format, a named stop per step); a row with no range is a checkbox.
local ARPG_OPTION_ROWS = {
    { "RowSeeThrough", "arpgSeeThrough", "See-Through Outdoors", "ARPG_TOOLTIP_SEE_THROUGH" },
    { "RowCameraDistance", "arpgCameraDistance", "Camera Distance", "ARPG_TOOLTIP_CAMERA_DISTANCE",
        5, 50, 1, "%d yd" },
    { "RowCameraPitch", "arpgCameraPitch", "Camera Pitch", "ARPG_TOOLTIP_CAMERA_PITCH",
        20, 89, 1, "%d deg" },
    { "RowCameraYaw", "arpgCameraYaw", "Camera Yaw", "ARPG_TOOLTIP_CAMERA_YAW",
        0, 360, 5, "%d deg" },
    { "RowCutHeight", "arpgCutHeight", "Cutaway Height", "ARPG_TOOLTIP_CUT_HEIGHT",
        1.5, 10, 0.1, "%.1f yd" },
    { "RowCutRadius", "arpgCutRadius", "Cutaway Radius", "ARPG_TOOLTIP_CUT_RADIUS",
        5, 120, 1, "%d yd" },
    { "RowLootFilter", "arpgLootFilter", "Loot Labels", "ARPG_TOOLTIP_LOOT_FILTER",
        0, 3, 1, nil, { "All", "No Grey", "Green and Better", "Blue and Better" } },
    { "RowDungeonTier", "arpgDungeonTier", "Dungeon Tier", "ARPG_TOOLTIP_DUNGEON_TIER",
        0, 5, 1, nil, { "Normal", "Hard", "Brutal", "Torment I", "Torment II", "Torment III" } },
    -- Developer: test loot, which a server with Arpg.DevTools = 1 drops at your feet.
    { "RowDevLootQuality", "arpgDevLootQuality", "Test Loot Quality", "ARPG_TOOLTIP_DEV_LOOT",
        0, 7, 1, nil, { "Mixed", "Grey", "White", "Green", "Blue", "Purple", "Unique", "Codex" } },
    { "RowDevLootCount", "arpgDevLootCount", "Test Loot Count", "ARPG_TOOLTIP_DEV_LOOT",
        1, 16, 1, "%d" },
    -- An item level, not a character's: past 60 it reaches the raids' gear (Molten Core from 66,
    -- Blackwing Lair about 75, Naxxramas up to 92), whose affixes roll larger.
    { "RowDevLootLevel", "arpgDevLootLevel", "Test Loot Item Level", "ARPG_TOOLTIP_DEV_LOOT",
        0, 92, 1, function(v)
            if v < 0.5 then return "Mine" end
            local n = math.floor(v + 0.5)
            if n >= 83 then return string.format("%d (Naxxramas)", n) end
            if n >= 71 then return string.format("%d (BWL, AQ)", n) end
            if n >= 66 then return string.format("%d (Molten Core)", n) end
            return string.format("%d", n)
        end },
}

local function addArpgOptionsPage()
    if not BenillaOptionsFrame or not BENILLA_OPTIONS_PAGE_ROWS
        or not BENILLA_OPTIONS_CATEGORY_KEYS or BENILLA_OPTIONS_PAGE_ROWS.Arpg then
        return
    end
    local list = BenillaOptionsFrameCategoryList
    local header = CreateFrame("Frame", "BenillaOptionsFrameCategoryListHeaderArpg", list,
        "BenillaOptionsCategoryHeaderTemplate")
    header:SetPoint("TOPLEFT", "BenillaOptionsFrameCategoryListRowAudio", "BOTTOMLEFT", 0, -22)
    getglobal(header:GetName() .. "Label"):SetText("ARPG")
    local entry = CreateFrame("Button", "BenillaOptionsFrameCategoryListRowArpg", list,
        "BenillaOptionsCategoryRowTemplate")
    entry:SetPoint("TOPLEFT", header, "BOTTOMLEFT", 0, -2)
    BenillaOptionsCategoryRow_OnLoad(entry, "Arpg", "ARPG View")

    local body = CreateFrame("Frame", "BenillaOptionsFrameContainerBodyArpg",
        BenillaOptionsFrameContainerBody)
    body:SetPoint("TOPLEFT", BenillaOptionsFrameContainerBody, "TOPLEFT", 0, 0)
    body:SetPoint("BOTTOMRIGHT", BenillaOptionsFrameContainerBody, "BOTTOMRIGHT", 0, 0)
    body:Hide()
    -- A search's heading over our matches, as the window has one per page.
    local head = CreateFrame("Button", "BenillaOptionsFrameContainerBodySearchHeadArpg",
        BenillaOptionsFrameContainerBody, "BenillaOptionsSearchHeadTemplate")
    head.categoryKey = "Arpg"
    getglobal(head:GetName() .. "Label"):SetText("ARPG View")

    local keys, prev = {}, nil
    for _, spec in ipairs(ARPG_OPTION_ROWS) do
        local key, cvar, label, tip, minv, maxv, step, fmt, stops =
            spec[1], spec[2], spec[3], spec[4], spec[5], spec[6], spec[7], spec[8], spec[9]
        local template = "BenillaOptionsCheckboxRowTemplate"
        if minv then template = "BenillaOptionsSliderRowTemplate" end
        local row = CreateFrame("Frame", body:GetName() .. key, body, template)
        if prev then
            row:SetPoint("TOPLEFT", prev, "BOTTOMLEFT", 0, -9)
            row:SetPoint("TOPRIGHT", prev, "BOTTOMRIGHT", 0, -9)
        else
            row:SetPoint("TOPLEFT", body, "TOPLEFT", 10, -12)
            row:SetPoint("TOPRIGHT", body, "TOPRIGHT", -20, -12)
        end
        BenillaOptionsRow_OnLoad(row, cvar, label, tip)
        if minv then
            BenillaOptionsSliderRow_Setup(row, minv, maxv, step, "arpg", stops)
            row.arpgFmt = fmt
        end
        table.insert(keys, key)
        prev = row
    end
    -- Drop Test Loot: a child of the last row, so a search hides it with that row. The client
    -- sends the request when the counter setting moves (arpgDevLootDrop, never saved).
    local drop = CreateFrame("Button", body:GetName() .. "DevLootDrop", prev,
        "BenillaOptionsRedButtonTemplate")
    drop:SetWidth(140)
    drop:SetPoint("TOPLEFT", prev, "BOTTOMLEFT", 37, -10)
    drop:SetText("Drop Test Loot")
    drop:SetScript("OnClick", function()
        PlaySound("igMainMenuOptionCheckBoxOn")
        SetCVar("arpgDevLootDrop", tostring((tonumber(GetCVar("arpgDevLootDrop")) or 0) + 1))
    end)
    -- Form Test Pack, Champion Pack, Rare Pack: the nearest mob forms a pack of five (Arpg.DevTools),
    -- through the web window's action setting (kind 9, the node field tier * 100 + size): tier 1
    -- plain, 2 with champions, 3 with a rare.
    local function packButton(key, label, tier, anchor, x, y)
        local b = CreateFrame("Button", body:GetName() .. key, prev, "BenillaOptionsRedButtonTemplate")
        b:SetWidth(140)
        b:SetPoint("TOPLEFT", anchor, "TOPLEFT", x, y)
        b:SetText(label)
        b:SetScript("OnClick", function()
            PlaySound("igMainMenuOptionCheckBoxOn")
            local n = (tonumber(GetCVar("arpgTreeAction")) or 0) + 1
            local nonce = n - math.floor(n / 100) * 100
            SetCVar("arpgTreeAction", tostring(900000 + (tier * 100 + 5) * 100 + nonce))
        end)
        return b
    end
    packButton("DevPack", "Form Test Pack", 1, drop, 150, 0)
    packButton("DevChampionPack", "Champion Pack", 2, drop, 0, -28)
    packButton("DevRarePack", "Rare Pack", 3, drop, 150, -28)

    BENILLA_OPTIONS_PAGE_ROWS.Arpg = keys
    table.insert(BENILLA_OPTIONS_CATEGORY_KEYS, "Arpg")

    -- The window's readouts know only its own formats; ours carry a printf pattern.
    local formatValue = BenillaOptionsRow_FormatValue
    BenillaOptionsRow_FormatValue = function(row, value)
        if type(row.arpgFmt) == "function" then return row.arpgFmt(value) end
        if row.arpgFmt then
            -- Whole-number readouts round rather than truncate.
            if string.find(row.arpgFmt, "%%d") then value = math.floor(value + 0.5) end
            return string.format(row.arpgFmt, value)
        end
        return formatValue(row, value)
    end
end

addArpgOptionsPage()

-- ── The ARPG passive web window ──
-- The server owns the web (Arpg/ArpgTree.h): the client hands each layout it sends to
-- ArpgTree_Update. The talent key and the talent micro button open this window in place of the
-- talent frame. Drag to pan, the wheel zooms; a click takes a node that joins the web, a right
-- click gives one back, Respec gives everything back (free, out of combat). Requests go back
-- through the session-only setting arpgTreeAction, as a number.

local WEB_REGION_RGB = {
    [0] = { 0.86, 0.36, 0.30 },  -- Crusader
    [1] = { 0.98, 0.84, 0.45 },  -- Lightbringer
    [2] = { 0.42, 0.62, 0.95 },  -- Templar
    [3] = { 0.80, 0.74, 0.60 },  -- the start and the bridges
}
local WEB_KIND_NAME = { [0] = "Start", [1] = "Small", [2] = "Notable", [3] = "Keystone" }
local WEB_SIZE = { [0] = 40, [1] = 20, [2] = 38, [3] = 50 }
local WEB_SOLID_TEX = "Interface\\Buttons\\WHITE8X8"
-- Paths are rows of small squares this far apart, in web units: solid at any zoom, and no
-- texture-coordinate tricks (a rotated route-line texture tiles outside its square).
local WEB_DOT_STEP = 8
local treeData, treeFrame, treeNonce = nil, nil, 0
local webById, webNext = {}, {}
-- The web spans about 1100 units each way, centred a little below the start.
local webView = { x = 0, y = 90, zoom = nil }

-- kind 1 take, 2 respec, 3 query, 4 give back: kind * 100000 + node * 100 + a nonce under 100.
local function treeAsk(kind, node)
    treeNonce = treeNonce + 1
    if treeNonce >= 100 then treeNonce = 1 end
    SetCVar("arpgTreeAction", tostring(kind * 100000 + (node or 0) * 100 + treeNonce))
end

-- The town portal (T, and the button beside the roll): out of combat, two seconds still open a
-- portal home; in town the same press goes back through it. The server answers in chat.
BINDING_HEADER_ARPG = "ARPG"
BINDING_NAME_ARPGTOWNPORTAL = "Town Portal"
function ArpgHud_TownPortal()
    treeAsk(12)
end

local portalButton = CreateFrame("Button", "ArpgHudPortalButton", UIParent)
portalButton:SetWidth(28)
portalButton:SetHeight(28)
portalButton:SetPoint("BOTTOMLEFT", UIParent, "BOTTOM", BAR_HALF + 14 + 70 + 10, BAR_FOOT + 2)
portalButton:SetFrameStrata("MEDIUM")
local portalIcon = portalButton:CreateTexture(nil, "ARTWORK")
portalIcon:SetAllPoints(portalButton)
portalIcon:SetTexture("Interface\\Icons\\Spell_Arcane_PortalStormWind")
local portalKey = portalButton:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
portalKey:SetPoint("BOTTOM", portalButton, "TOP", 0, 2)
-- The key over the button: whatever the town portal is bound to, none shown when unbound.
local function showPortalKey()
    local key = GetBindingKey and GetBindingKey("ARPGTOWNPORTAL")
    portalKey:SetText(key or "")
end
portalButton:SetScript("OnClick", function() ArpgHud_TownPortal() end)
portalButton:SetScript("OnEnter", function()
    if GameTooltip then
        GameTooltip:SetOwner(this, "ANCHOR_TOP")
        GameTooltip:SetText("Town Portal")
        GameTooltip:AddLine("Out of combat, stand still for two seconds to open a portal home. "
            .. "In town, use it again within 30 minutes to go back.", 1, 1, 1, 1)
        GameTooltip:Show()
    end
end)
portalButton:SetScript("OnLeave", function()
    if GameTooltip then
        GameTooltip:Hide()
    end
end)

-- T opens the portal unless the player has bound T to something else; set once, on login.
local portalBinder = CreateFrame("Frame")
portalBinder:RegisterEvent("PLAYER_ENTERING_WORLD")
portalBinder:SetScript("OnEvent", function()
    this:UnregisterEvent("PLAYER_ENTERING_WORLD")
    if not GetBindingKey or not SetBinding then
        return
    end
    if GetBindingKey("ARPGTOWNPORTAL") then
        showPortalKey()
        return
    end
    local taken = GetBindingAction and GetBindingAction("T")
    if taken == nil or taken == "" then
        if SetBinding("T", "ARPGTOWNPORTAL") and SaveBindings and GetCurrentBindingSet then
            SaveBindings(GetCurrentBindingSet())
        end
    end
    showPortalKey()
end)
local portalKeyWatch = CreateFrame("Frame")
portalKeyWatch:RegisterEvent("UPDATE_BINDINGS")
portalKeyWatch:SetScript("OnEvent", showPortalKey)

local function webTaken(id)
    local n = webById[id]
    return n and n.taken == 1
end

-- Whether `n` joins a taken node.
local function webJoined(n)
    for _, m in ipairs(webNext[n.id] or {}) do
        if webTaken(m) then return true end
    end
    return false
end

-- Why `n` cannot be taken now, or nil when it can.
local function webBlocked(n)
    if n.taken == 1 then return "Taken" end
    if not webJoined(n) then return "Not joined to your web yet" end
    if treeData.spent >= treeData.total then return "No points left" end
    return nil
end

-- A path is `n` squares in `line` (a table of textures, grown as needed), on `parent`.
local function lineDots(line, parent, layer, n)
    for k = 1, n do
        if not line[k] then
            local T = parent:CreateTexture(nil, layer)
            T:SetTexture(WEB_SOLID_TEX)
            line[k] = T
        end
    end
    for k = n + 1, table.getn(line) do line[k]:Hide() end
end

-- Lay `line`'s first `n` squares evenly from (ax, ay) to (bx, by), from `parent`'s TOPLEFT, y down.
local function placeDots(line, parent, n, ax, ay, bx, by, size)
    for k = 1, n do
        local t = (k - 0.5) / n
        local T = line[k]
        T:ClearAllPoints()
        T:SetWidth(size)
        T:SetHeight(size)
        T:SetPoint("CENTER", parent, "TOPLEFT", ax + (bx - ax) * t, -(ay + (by - ay) * t))
        T:Show()
    end
end

local function colorDots(line, n, r, g, b, a)
    for k = 1, n do line[k]:SetVertexColor(r, g, b, a) end
end

local function webShowTip(button)
    local n = button.node
    if not n then return end
    local rgb = WEB_REGION_RGB[n.region] or WEB_REGION_RGB[3]
    GameTooltip:SetOwner(button, "ANCHOR_RIGHT")
    GameTooltip:SetText(n.name, rgb[1], rgb[2], rgb[3])
    GameTooltip:AddLine(WEB_KIND_NAME[n.kind] or "", 0.6, 0.6, 0.6)
    GameTooltip:AddLine(n.text, 1, 0.82, 0, 1)
    if n.kind ~= 0 then
        local blocked = webBlocked(n)
        if n.taken == 1 then
            GameTooltip:AddLine("Right-click to give it back", 0.5, 0.8, 1)
        elseif blocked then
            GameTooltip:AddLine(blocked, 1, 0.13, 0.13)
        else
            GameTooltip:AddLine("Click to take", 0, 1, 0)
        end
    end
    GameTooltip:Show()
end

local function webMakeButton(parent, id)
    local b = CreateFrame("Button", "ArpgWebNode" .. id, parent)
    b.border = b:CreateTexture(nil, "BACKGROUND")
    b.icon = b:CreateTexture(nil, "ARTWORK")
    b.icon:SetAllPoints(b)
    b.glow = b:CreateTexture(nil, "OVERLAY")
    b.glow:SetTexture("Interface\\Buttons\\ButtonHilight-Square")
    b.glow:SetBlendMode("ADD")
    b.glow:SetAllPoints(b)
    b.glow:Hide()
    b:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    b:SetScript("OnEnter", function() this.glow:Show(); webShowTip(this) end)
    b:SetScript("OnLeave", function() this.glow:Hide(); GameTooltip:Hide() end)
    b:SetScript("OnClick", function()
        local n = this.node
        if not n or n.kind == 0 then return end
        if arg1 == "RightButton" then
            if n.taken == 1 then treeAsk(4, n.id) end
        elseif not webBlocked(n) then
            treeAsk(1, n.id)
        end
    end)
    return b
end

-- Where web point (wx, wy) sits on the canvas, from its TOPLEFT, y down.
local function webToCanvas(wx, wy)
    local f = treeFrame
    return f.canvasW / 2 + (wx - webView.x) * webView.zoom, f.canvasH / 2 + (wy - webView.y) * webView.zoom
end

-- Lay every node, line and label out at the current pan and zoom.
local function webLayout()
    local f = treeFrame
    if not f or not treeData then return end
    local canvas, z = f.canvas, webView.zoom
    for _, n in ipairs(treeData.nodes) do
        local b = f.buttons[n.id]
        if b then
            local px, py = webToCanvas(n.x, n.y)
            local size = math.max(8, (WEB_SIZE[n.kind] or 16) * math.max(0.6, z * 1.6))
            b:SetWidth(size)
            b:SetHeight(size)
            b:ClearAllPoints()
            b:SetPoint("CENTER", canvas, "TOPLEFT", px, -py)
            local pad = n.kind == 1 and 1 or math.max(2, size * 0.08)
            b.border:ClearAllPoints()
            b.border:SetPoint("TOPLEFT", b, "TOPLEFT", -pad, pad)
            b.border:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", pad, -pad)
        end
    end
    local size = math.max(2, WEB_DOT_STEP * z * 1.05)
    for i, link in ipairs(treeData.links) do
        local line = f.lines[i]
        local a, b = webById[link[1]], webById[link[2]]
        if line and a and b then
            local ax, ay = webToCanvas(a.x, a.y)
            local bx, by = webToCanvas(b.x, b.y)
            placeDots(line, canvas, f.lineN[i], ax, ay, bx, by, size)
        end
    end
    for i, region in ipairs(treeData.regions) do
        local label = f.labels[i]
        if label then
            local px, py = webToCanvas(region.x, region.y)
            label:ClearAllPoints()
            label:SetPoint("CENTER", canvas, "TOPLEFT", px, -py)
        end
    end
end

-- Colour every node and line for what is taken and what can be.
local function webPaint()
    local f = treeFrame
    for _, n in ipairs(treeData.nodes) do
        local b = f.buttons[n.id]
        local rgb = WEB_REGION_RGB[n.region] or WEB_REGION_RGB[3]
        local taken = n.taken == 1
        local open = not taken and not webBlocked(n)
        if n.kind == 1 then
            b.icon:SetTexture(WEB_SOLID_TEX)
            b.icon:SetDesaturated(nil)
            b.border:SetTexture(0, 0, 0, 0.9)
            if taken then
                b.icon:SetVertexColor(rgb[1], rgb[2], rgb[3])
            elseif open then
                b.icon:SetVertexColor(0.85, 0.85, 0.85)
            else
                b.icon:SetVertexColor(0.32, 0.32, 0.34)
            end
        else
            b.icon:SetTexture(n.icon ~= "" and n.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
            b.icon:SetDesaturated(not (taken or open) and 1 or nil)
            if taken then
                b.icon:SetVertexColor(1, 1, 1)
                b.border:SetTexture(rgb[1], rgb[2], rgb[3], 1)
            elseif open then
                b.icon:SetVertexColor(0.9, 0.9, 0.9)
                b.border:SetTexture(0.75, 0.75, 0.75, 0.9)
            else
                b.icon:SetVertexColor(0.55, 0.55, 0.55)
                b.border:SetTexture(0.18, 0.18, 0.2, 0.95)
            end
            if n.kind == 3 and taken then
                b.border:SetTexture(1, 0.5, 0, 1)
            end
        end
        b.node = n
        b:Show()
    end
    for i, link in ipairs(treeData.links) do
        local line, n = f.lines[i], f.lineN[i]
        local a, b = webTaken(link[1]), webTaken(link[2])
        if a and b then
            colorDots(line, n, 1, 0.8, 0.3, 1)
        elseif a or b then
            colorDots(line, n, 0.6, 0.6, 0.62, 0.9)
        else
            colorDots(line, n, 0.28, 0.28, 0.32, 0.8)
        end
    end
    for i, region in ipairs(treeData.regions) do
        local label = f.labels[i]
        local rgb = WEB_REGION_RGB[i - 1] or WEB_REGION_RGB[3]
        label:SetText(region.name)
        label:SetTextColor(rgb[1], rgb[2], rgb[3], 0.9)
    end
    if f.tab ~= "skills" then
        f.points:SetText("Points: " .. (treeData.total - treeData.spent) .. " of " .. treeData.total .. " left")
    end
end

-- Zoom by `factor` about canvas point (cx, cy), from its TOPLEFT, y down.
local function webZoom(factor, cx, cy)
    local f = treeFrame
    local old = webView.zoom
    local new = math.max(0.25, math.min(1.6, old * factor))
    -- The web point under (cx, cy) stays under it.
    local wx = webView.x + (cx - f.canvasW / 2) / old
    local wy = webView.y + (cy - f.canvasH / 2) / old
    webView.zoom = new
    webView.x = wx - (cx - f.canvasW / 2) / new
    webView.y = wy - (cy - f.canvasH / 2) / new
    webLayout()
end

-- The cursor on the canvas, from its TOPLEFT, y down.
local function webCursor()
    local f = treeFrame
    local x, y = GetCursorPosition()
    local scale = f.canvas:GetEffectiveScale()
    return x / scale - f.view:GetLeft(), f.view:GetTop() - y / scale
end

local function treeBuild()
    if treeFrame then return treeFrame end
    local f = CreateFrame("Frame", "ArpgTreeFrame", UIParent)
    local width = math.min(1000, UIParent:GetWidth() - 60)
    local height = math.min(800, UIParent:GetHeight() - 80)
    f:SetWidth(width)
    f:SetHeight(height)
    f:SetPoint("CENTER", UIParent, "CENTER", 0, 30)
    f:SetFrameStrata("DIALOG")
    f:EnableMouse(true)
    f:Hide()
    local bg = f:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(f)
    bg:SetTexture(0.03, 0.028, 0.04, 0.96)
    local edge = f:CreateTexture(nil, "BORDER")
    edge:SetPoint("TOPLEFT", f, "TOPLEFT", 0, -36)
    edge:SetPoint("TOPRIGHT", f, "TOPRIGHT", 0, -36)
    edge:SetHeight(1)
    edge:SetTexture(0.9, 0.8, 0.5, 0.35)
    f.title = f:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
    f.title:SetPoint("TOP", f, "TOP", 0, -10)
    f.title:SetText("Passive Web")
    f.points = f:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    f.points:SetPoint("TOPLEFT", f, "TOPLEFT", 16, -13)
    local close = CreateFrame("Button", "ArpgTreeFrameClose", f, "UIPanelCloseButton")
    close:SetPoint("TOPRIGHT", f, "TOPRIGHT", -2, -2)
    -- Two tabs: the passive web and the specialised skills.
    f.tabs = {}
    for i, tab in ipairs({ { "web", "Passive Web" }, { "skills", "Skills" } }) do
        local b = CreateFrame("Button", "ArpgTreeFrameTab" .. i, f, "UIPanelButtonTemplate")
        b:SetWidth(100)
        b:SetHeight(20)
        b:SetPoint("TOPRIGHT", f, "TOPRIGHT", -36 - (2 - i) * 104, -8)
        b:SetText(tab[2])
        b.tab = tab[1]
        b:SetScript("OnClick", function() ArpgTree_ShowTab(this.tab) end)
        f.tabs[i] = b
    end
    f.webPane = CreateFrame("Frame", "ArpgTreeWebPane", f)
    f.webPane:SetAllPoints(f)
    local hint = f.webPane:CreateFontString(nil, "OVERLAY", "GameFontDisableSmall")
    hint:SetPoint("BOTTOMLEFT", f, "BOTTOMLEFT", 16, 14)
    hint:SetText("Drag to move, mouse wheel to zoom. Click a node joined to your web to take it; right-click to give one back.")
    local respec = CreateFrame("Button", "ArpgTreeFrameRespec", f.webPane, "UIPanelButtonTemplate")
    respec:SetWidth(90)
    respec:SetHeight(22)
    respec:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -12, 9)
    respec:SetText("Respec")
    respec:SetScript("OnClick", function() treeAsk(2) end)
    -- The character's ARPG numbers, from the server's sheet (ArpgSheet_Update).
    local totals = CreateFrame("Button", "ArpgTreeFrameTotals", f.webPane, "UIPanelButtonTemplate")
    totals:SetWidth(90)
    totals:SetHeight(22)
    totals:SetPoint("RIGHT", respec, "LEFT", -6, 0)
    totals:SetText("Totals")
    totals:SetScript("OnEnter", function() ArpgSheet_SummaryTip(this) end)
    totals:SetScript("OnLeave", function() GameTooltip:Hide() end)

    -- The viewport clips the canvas, which holds the lines, labels and node buttons.
    local view = CreateFrame("ScrollFrame", "ArpgTreeView", f.webPane)
    view:SetPoint("TOPLEFT", f, "TOPLEFT", 4, -38)
    view:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -4, 38)
    f.canvasW, f.canvasH = width - 8, height - 76
    local canvas = CreateFrame("Frame", "ArpgTreeCanvas", view)
    canvas:SetWidth(f.canvasW)
    canvas:SetHeight(f.canvasH)
    view:SetScrollChild(canvas)
    view:EnableMouse(true)
    view:EnableMouseWheel(true)
    view:SetScript("OnMouseWheel", function()
        local cx, cy = webCursor()
        webZoom(arg1 > 0 and 1.15 or 1 / 1.15, cx, cy)
    end)
    view:SetScript("OnMouseDown", function()
        local cx, cy = webCursor()
        f.drag = { cx = cx, cy = cy, x = webView.x, y = webView.y }
    end)
    view:SetScript("OnMouseUp", function() f.drag = nil end)
    view:SetScript("OnUpdate", function()
        local d = f.drag
        if not d then return end
        local cx, cy = webCursor()
        webView.x = d.x - (cx - d.cx) / webView.zoom
        webView.y = d.y - (cy - d.cy) / webView.zoom
        webLayout()
    end)
    f.view, f.canvas = view, canvas
    f.buttons, f.lines, f.lineN, f.labels = {}, {}, {}, {}
    f:SetScript("OnHide", function() f.drag = nil end)
    if UISpecialFrames then tinsert(UISpecialFrames, "ArpgTreeFrame") end
    treeFrame = f
    return f
end

-- Take a new layout: index it, make what it needs, then paint and lay it out.
local function treeRedraw()
    if not treeData then return end
    local f = treeBuild()
    webById, webNext = {}, {}
    for _, n in ipairs(treeData.nodes) do
        webById[n.id] = n
        webNext[n.id] = {}
        if not f.buttons[n.id] then
            f.buttons[n.id] = webMakeButton(f.canvas, n.id)
        end
    end
    for i, link in ipairs(treeData.links) do
        if webNext[link[1]] and webNext[link[2]] then
            tinsert(webNext[link[1]], link[2])
            tinsert(webNext[link[2]], link[1])
        end
        local a, b = webById[link[1]], webById[link[2]]
        local length = (a and b) and math.sqrt((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y)) or 0
        f.lineN[i] = math.max(1, math.ceil(length / WEB_DOT_STEP))
        f.lines[i] = f.lines[i] or {}
        lineDots(f.lines[i], f.canvas, "BACKGROUND", f.lineN[i])
    end
    for i = table.getn(treeData.links) + 1, table.getn(f.lines) do
        lineDots(f.lines[i], f.canvas, "BACKGROUND", 0)
        f.lineN[i] = 0
    end
    for i, _ in ipairs(treeData.regions) do
        if not f.labels[i] then
            f.labels[i] = f.canvas:CreateFontString(nil, "ARTWORK", "GameFontNormalLarge")
        end
    end
    for id, b in pairs(f.buttons) do
        if not webById[id] then b:Hide() end
    end
    -- A class with no web yet says so, rather than an empty canvas.
    if not f.emptyNote then
        f.emptyNote = f.webPane:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
        f.emptyNote:SetPoint("CENTER", f.webPane, "CENTER", 0, 0)
        f.emptyNote:SetText("Your class has no passive web yet.")
    end
    if table.getn(treeData.nodes) == 0 then f.emptyNote:Show() else f.emptyNote:Hide() end
    if not webView.zoom then
        -- First open: the whole web in view.
        webView.zoom = math.min(f.canvasW, f.canvasH) / 1200
    end
    webPaint()
    webLayout()
end

-- ── The Skills tab ──
-- The server owns the skills (Arpg/ArpgSkills.h) and hands them to ArpgSkills_Update. Five slots
-- open by level; a skill in a slot takes points in its tree (click a node, right-click to give a
-- rank back). Taking a skill out of its slot gives its points back.

local SKILL_KIND_NAME = { [1] = "Modifier", [2] = "Transformer", [3] = "Synergy", [4] = "Capstone" }
local SKILL_KIND_RGB = {
    [1] = { 0.55, 0.65, 0.80 },
    [2] = { 0.902, 0.8, 0.502 },
    [3] = { 0.35, 0.80, 0.75 },
    [4] = { 1.0, 0.5, 0.0 },
}
local SKILL_COL_W, SKILL_ROW_H, SKILL_NODE = 150, 92, 40
local skillsData, skillSel, slotSel = nil, nil, 1

local function skillById(id)
    if not skillsData then return nil end
    for _, k in ipairs(skillsData.skills) do
        if k.id == id then return k end
    end
    return nil
end

-- The slot a skill sits in, or nil.
local function skillSlot(id)
    for i, slot in ipairs(skillsData.slots) do
        if slot.skill == id then return i end
    end
    return nil
end

local function skillRank(k, id)
    if id == 0 then return 1 end
    for _, n in ipairs(k.nodes) do
        if n.id == id then return n.rank end
    end
    return 0
end

-- Whether bit `bit` of `mask` is set (Lua 5.0 has no bit operators).
local function bitSet(mask, bit)
    local m = mask or 0
    for _ = 1, bit do m = math.floor(m / 2) end
    return m - math.floor(m / 2) * 2 == 1
end

-- Why a rank cannot go into node `n` of skill `k` now, or nil when it can.
local function skillBlocked(k, n)
    if not skillSlot(k.id) then return "Specialise " .. k.name .. " in a slot first" end
    if n.sealed then return "Sealed: read its Codex page first" end
    if n.rank >= n.max then return "Fully learned" end
    if skillRank(k, n.parent) == 0 then return "Needs a rank in the node above" end
    if k.spent >= k.cap then return k.name .. " has all " .. k.cap .. " points it can take" end
    if skillsData.spent >= skillsData.total then return "No skill points left" end
    return nil
end

local function skillShowTip(button)
    local n, k = button.node, button.skill
    if not n or not k then return end
    GameTooltip:SetOwner(button, "ANCHOR_RIGHT")
    GameTooltip:SetText(n.name, 1, 1, 1)
    local rgb = SKILL_KIND_RGB[n.kind] or SKILL_KIND_RGB[1]
    GameTooltip:AddLine((SKILL_KIND_NAME[n.kind] or "") .. "   Rank " .. n.rank .. "/" .. n.max, rgb[1], rgb[2], rgb[3])
    GameTooltip:AddLine(n.text, 1, 0.82, 0, 1)
    if n.sealed then
        GameTooltip:AddLine("Sealed", 0.64, 0.21, 0.93)
        if n.home and n.home ~= "" then
            GameTooltip:AddLine("Its Codex page drops from " .. n.home .. ".", 0.8, 0.8, 0.8, 1)
        end
        local have, need = skillsData.fragments or 0, skillsData.perPage or 5
        if have >= need and skillSlot(k.id) then
            GameTooltip:AddLine("Shift-click to unseal it with " .. need .. " Codex fragments (you have " .. have .. ")", 0, 1, 0, 1)
        else
            GameTooltip:AddLine(need .. " Codex fragments also unseal it (you have " .. have .. ")", 0.6, 0.6, 0.6, 1)
        end
    end
    local blocked = skillBlocked(k, n)
    if blocked then
        GameTooltip:AddLine(blocked, 1, 0.13, 0.13)
    else
        GameTooltip:AddLine("Click to learn", 0, 1, 0)
    end
    if n.rank > 0 then GameTooltip:AddLine("Right-click to give a rank back", 0.5, 0.8, 1) end
    GameTooltip:Show()
end

local function skillMakeNode(parent, i)
    local b = CreateFrame("Button", "ArpgSkillNode" .. i, parent)
    b.border = b:CreateTexture(nil, "BACKGROUND")
    b.border:SetPoint("TOPLEFT", b, "TOPLEFT", -3, 3)
    b.border:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", 3, -3)
    b.icon = b:CreateTexture(nil, "ARTWORK")
    b.icon:SetAllPoints(b)
    b.rank = b:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    b.rank:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", 4, -4)
    b.label = b:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
    b.label:SetPoint("TOP", b, "BOTTOM", 0, -5)
    b:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    b:SetScript("OnEnter", function() skillShowTip(this) end)
    b:SetScript("OnLeave", function() GameTooltip:Hide() end)
    b:SetScript("OnClick", function()
        local n, k = this.node, this.skill
        if not n or not k or n.kind == 0 then return end
        if n.sealed then
            if IsShiftKeyDown() and skillSlot(k.id) and (skillsData.fragments or 0) >= (skillsData.perPage or 5) then
                treeAsk(10, n.id)
            end
            return
        end
        if arg1 == "RightButton" then
            if n.rank > 0 then treeAsk(7, n.id) end
        elseif not skillBlocked(k, n) then
            treeAsk(6, n.id)
        end
    end)
    return b
end

local function skillMakeRow(parent, name, width)
    local b = CreateFrame("Button", name, parent)
    b:SetWidth(width)
    b:SetHeight(36)
    b.bg = b:CreateTexture(nil, "BACKGROUND")
    b.bg:SetAllPoints(b)
    b.icon = b:CreateTexture(nil, "ARTWORK")
    b.icon:SetWidth(30)
    b.icon:SetHeight(30)
    b.icon:SetPoint("LEFT", b, "LEFT", 3, 0)
    b.text = b:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    b.text:SetPoint("TOPLEFT", b, "TOPLEFT", 40, -4)
    b.text:SetJustifyH("LEFT")
    b.sub = b:CreateFontString(nil, "OVERLAY", "GameFontDisableSmall")
    b.sub:SetPoint("BOTTOMLEFT", b, "BOTTOMLEFT", 40, 5)
    b.sub:SetJustifyH("LEFT")
    b.glow = b:CreateTexture(nil, "HIGHLIGHT")
    b.glow:SetAllPoints(b)
    b.glow:SetTexture(1, 1, 1, 0.08)
    return b
end

local function skillsBuild(f)
    if f.skillPane then return f.skillPane end
    local p = CreateFrame("Frame", "ArpgSkillPane", f)
    p:SetPoint("TOPLEFT", f, "TOPLEFT", 4, -38)
    p:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -4, 4)
    p:Hide()
    local slotsLabel = p:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    slotsLabel:SetPoint("TOPLEFT", p, "TOPLEFT", 12, -8)
    slotsLabel:SetText("Slots")
    p.slots = {}
    for i = 1, 5 do
        local row = skillMakeRow(p, "ArpgSkillSlot" .. i, 240)
        row:SetPoint("TOPLEFT", p, "TOPLEFT", 10, -26 - (i - 1) * 40)
        row.index = i
        row:SetScript("OnClick", function()
            slotSel = this.index
            local slot = skillsData and skillsData.slots[this.index]
            if slot and slot.skill ~= 0 then skillSel = slot.skill end
            ArpgSkills_Redraw()
        end)
        p.slots[i] = row
    end
    local skillsLabel = p:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    skillsLabel:SetPoint("TOPLEFT", p, "TOPLEFT", 12, -232)
    skillsLabel:SetText("Skills")
    p.rows = {}
    for i = 1, 8 do
        local row = skillMakeRow(p, "ArpgSkillRow" .. i, 240)
        row:SetPoint("TOPLEFT", p, "TOPLEFT", 10, -250 - (i - 1) * 40)
        row:SetScript("OnClick", function()
            skillSel = this.skillId
            ArpgSkills_Redraw()
        end)
        p.rows[i] = row
    end
    local function button(name, label, x)
        local b = CreateFrame("Button", name, p, "UIPanelButtonTemplate")
        b:SetWidth(110)
        b:SetHeight(22)
        b:SetPoint("BOTTOMRIGHT", p, "BOTTOMRIGHT", x, 6)
        b:SetText(label)
        return b
    end
    -- Put the shown skill in the chosen slot (or the first open, empty one).
    p.put = button("ArpgSkillPut", "Specialise", -244)
    p.put:SetScript("OnClick", function()
        if not skillsData or not skillSel then return end
        local slot = skillsData.slots[slotSel]
        local target = nil
        if slot and slot.level <= skillsData.level then target = slotSel end
        if not target or (skillsData.slots[target].skill ~= 0 and skillsData.slots[target].skill ~= skillSel) then
            for i, s in ipairs(skillsData.slots) do
                if s.skill == 0 and s.level <= skillsData.level then target = i break end
            end
        end
        if target then treeAsk(5, (target - 1) * 100 + skillSel) end
    end)
    p.take = button("ArpgSkillTake", "Take Out", -128)
    p.take:SetScript("OnClick", function()
        local slot = skillSel and skillSlot(skillSel)
        if slot then treeAsk(5, (slot - 1) * 100) end
    end)
    p.respec = button("ArpgSkillRespec", "Respec Skill", -12)
    p.respec:SetScript("OnClick", function()
        if skillSel then treeAsk(8, skillSel) end
    end)
    -- What adds to the shown skill: its tags' bonuses by source, its nodes, rune and uniques.
    p.bonus = button("ArpgSkillBonus", "Bonuses", -360)
    p.bonus:SetScript("OnEnter", function() ArpgSkillBonus_Tip(this) end)
    p.bonus:SetScript("OnLeave", function() GameTooltip:Hide() end)
    -- The tree, right of the lists.
    p.tree = CreateFrame("Frame", "ArpgSkillTree", p)
    p.tree:SetPoint("TOPLEFT", p, "TOPLEFT", 270, -8)
    p.tree:SetPoint("BOTTOMRIGHT", p, "BOTTOMRIGHT", -8, 36)
    local shade = p.tree:CreateTexture(nil, "BACKGROUND")
    shade:SetAllPoints(p.tree)
    shade:SetTexture(1, 1, 1, 0.03)
    p.treeTitle = p.tree:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
    p.treeTitle:SetPoint("TOPLEFT", p.tree, "TOPLEFT", 12, -10)
    p.treeText = p.tree:CreateFontString(nil, "OVERLAY", "GameFontHighlightSmall")
    p.treeText:SetPoint("TOPLEFT", p.tree, "TOPLEFT", 12, -32)
    p.treeText:SetJustifyH("LEFT")
    p.nodes, p.lines, p.branches = {}, {}, {}
    -- The Codex fragments held, and the shown skill's rune socket.
    p.fragments = p.tree:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
    p.fragments:SetPoint("TOPRIGHT", p.tree, "TOPRIGHT", -12, -12)
    p.socket = CreateFrame("Button", "ArpgRuneSocket", p.tree)
    p.socket:SetWidth(36)
    p.socket:SetHeight(36)
    p.socket:SetPoint("TOPRIGHT", p.tree, "TOPRIGHT", -14, -32)
    p.socket.ring = p.socket:CreateTexture(nil, "BACKGROUND")
    p.socket.ring:SetPoint("TOPLEFT", p.socket, "TOPLEFT", -3, 3)
    p.socket.ring:SetPoint("BOTTOMRIGHT", p.socket, "BOTTOMRIGHT", 3, -3)
    p.socket.icon = p.socket:CreateTexture(nil, "ARTWORK")
    p.socket.icon:SetAllPoints(p.socket)
    p.socket.label = p.socket:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
    p.socket.label:SetPoint("RIGHT", p.socket, "LEFT", -8, 0)
    p.socket:SetScript("OnEnter", function() ArpgRuneSocket_Tip(this) end)
    p.socket:SetScript("OnLeave", function() GameTooltip:Hide() end)
    p.socket:SetScript("OnClick", function() ArpgRuneSocket_Menu(this) end)
    p.runeMenu = CreateFrame("Frame", "ArpgRuneMenu", p, "UIDropDownMenuTemplate")
    f.skillPane = p
    return p
end

local function runeById(id)
    if not skillsData or not skillsData.runes then return nil end
    for _, r in ipairs(skillsData.runes) do
        if r.id == id then return r end
    end
    return nil
end

-- How many of rune `id` are free to socket in skill `skillId`: held, less those in other skills.
local function runeFree(id, skillId)
    local r = runeById(id)
    if not r then return 0 end
    local used = 0
    for _, k in ipairs(skillsData.skills) do
        if k.rune == id and k.id ~= skillId then used = used + 1 end
    end
    return r.held - used
end

function ArpgRuneSocket_Tip(button)
    local k = skillsData and skillById(skillSel)
    if not k then return end
    GameTooltip:SetOwner(button, "ANCHOR_LEFT")
    GameTooltip:SetText("Rune Socket", 1, 1, 1)
    local r = runeById(k.rune)
    if r then
        GameTooltip:AddLine(r.name, 0, 0.44, 0.87)
        GameTooltip:AddLine(r.text, 1, 0.82, 0, 1)
    elseif k.spent < (k.socketAt or 10) then
        GameTooltip:AddLine("Opens with " .. (k.socketAt or 10) .. " points in " .. k.name .. ".", 0.8, 0.8, 0.8, 1)
    else
        GameTooltip:AddLine("Empty. Runes drop from champions, rares and bosses.", 0.8, 0.8, 0.8, 1)
    end
    if skillSlot(k.id) and k.spent >= (k.socketAt or 10) then
        GameTooltip:AddLine("Click to choose a rune", 0, 1, 0)
    end
    GameTooltip:Show()
end

function ArpgRuneSocket_Menu(button)
    local k = skillsData and skillById(skillSel)
    if not k or not skillSlot(k.id) or k.spent < (k.socketAt or 10) then return end
    local menu = getglobal("ArpgRuneMenu")
    UIDropDownMenu_Initialize(menu, function()
        local any = false
        for _, r in ipairs(skillsData.runes or {}) do
            if bitSet(r.fits, k.id) and runeFree(r.id, k.id) > 0 then
                local info = {}
                local id = r.id
                info.text = r.name .. " (" .. runeFree(r.id, k.id) .. ")"
                info.checked = k.rune == r.id
                info.func = function() treeAsk(11, k.id * 100 + id) end
                UIDropDownMenu_AddButton(info)
                any = true
            end
        end
        if k.rune ~= 0 then
            local info = {}
            info.text = "Empty the socket"
            info.func = function() treeAsk(11, k.id * 100) end
            UIDropDownMenu_AddButton(info)
        elseif not any then
            local info = {}
            info.text = "No rune that fits " .. k.name
            info.disabled = 1
            UIDropDownMenu_AddButton(info)
        end
    end, "MENU")
    ToggleDropDownMenu(1, nil, menu, button:GetName(), 0, 0)
end

local function skillsPaint()
    local f = treeFrame
    if not f or not skillsData then return end
    local p = skillsBuild(f)
    if f.tab == "skills" then
        f.points:SetText("Skill points: " .. (skillsData.total - skillsData.spent) .. " of " .. skillsData.total .. " left")
    end
    if not p.emptyNote then
        p.emptyNote = p:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
        p.emptyNote:SetPoint("CENTER", p, "CENTER", 0, 0)
        p.emptyNote:SetText("Your class has no skill trees yet.")
    end
    if table.getn(skillsData.skills) == 0 then p.emptyNote:Show() else p.emptyNote:Hide() end
    if not skillSel or not skillById(skillSel) then
        local first = skillsData.skills[1]
        skillSel = first and first.id
    end
    for i, row in ipairs(p.slots) do
        local slot = skillsData.slots[i]
        if slot then
            local k = skillById(slot.skill)
            local open = slot.level <= skillsData.level
            if k then
                row.icon:SetTexture(k.icon ~= "" and k.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
                row.text:SetText(k.name)
                row.sub:SetText("Slot " .. i .. "   " .. k.spent .. "/" .. k.cap .. " points")
            else
                row.icon:SetTexture("Interface\\Buttons\\UI-EmptySlot")
                row.text:SetText(open and "Empty" or "Locked")
                row.sub:SetText(open and ("Slot " .. i) or ("Opens at level " .. slot.level))
            end
            row.icon:SetDesaturated(not open and 1 or nil)
            row.bg:SetTexture(1, 0.82, 0, i == slotSel and 0.18 or 0.05)
            row:Show()
        else
            row:Hide()
        end
    end
    for i, row in ipairs(p.rows) do
        local k = skillsData.skills[i]
        if k then
            row.skillId = k.id
            row.icon:SetTexture(k.icon ~= "" and k.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
            row.text:SetText(k.name)
            local slot = skillSlot(k.id)
            row.sub:SetText(slot and ("In slot " .. slot) or "Not specialised")
            row.bg:SetTexture(1, 1, 1, k.id == skillSel and 0.16 or 0.04)
            row:Show()
        else
            row:Hide()
        end
    end

    -- The shown skill's tree: the root on top, a column per branch, a row per depth.
    local k = skillById(skillSel)
    for _, b in pairs(p.nodes) do b:Hide() end
    for _, line in pairs(p.lines) do lineDots(line, p.tree, "BORDER", 0) end
    for _, l in pairs(p.branches) do l:Hide() end
    if not k then
        p.socket:Hide()
        return
    end
    p.treeTitle:SetText(k.name .. "   " .. k.spent .. "/" .. k.cap)
    p.treeText:SetText(k.text)
    p.fragments:SetText("Codex fragments: " .. (skillsData.fragments or 0))
    local rune = runeById(k.rune)
    local open = skillSlot(k.id) and k.spent >= (k.socketAt or 10)
    if rune then
        p.socket.icon:SetTexture(rune.icon ~= "" and rune.icon or "Interface\\Icons\\INV_Misc_Rune_01")
        p.socket.ring:SetTexture(0, 0.44, 0.87, 0.95)
        p.socket.label:SetText(rune.name)
    else
        p.socket.icon:SetTexture("Interface\\Buttons\\UI-EmptySlot")
        p.socket.ring:SetTexture(0.3, 0.3, 0.35, open and 0.9 or 0.4)
        p.socket.label:SetText(open and "Rune socket" or ("Socket at " .. (k.socketAt or 10) .. " points"))
    end
    p.socket.icon:SetDesaturated(not open and 1 or nil)
    p.socket:Show()
    -- The tree's size from the window's own (set, not anchored), as its layout may not have run yet.
    -- The rows share the height below the title, however deep the tree runs.
    local width, height = f:GetWidth() - 286, f:GetHeight() - 86
    local left = (width - 2 * SKILL_COL_W) / 2
    local deepest = 1
    for _, n in ipairs(k.nodes) do
        if n.row > deepest then deepest = n.row end
    end
    local rowH = math.min(SKILL_ROW_H, math.floor((height - 92 - 44) / deepest))
    local function at(col, row)
        return left + col * SKILL_COL_W, 92 + row * rowH
    end
    local all = { { id = 0, kind = 0, col = 1, row = 0, parent = -1, max = 0, rank = 1, icon = k.icon, name = k.name, text = k.text } }
    for _, n in ipairs(k.nodes) do tinsert(all, n) end
    local pos = {}
    for i, n in ipairs(all) do
        local b = p.nodes[i]
        if not b then
            b = skillMakeNode(p.tree, i)
            p.nodes[i] = b
        end
        local x, y = at(n.col, n.row)
        pos[n.id] = { x, y }
        local size = n.kind == 4 and SKILL_NODE + 8 or (n.kind == 0 and SKILL_NODE + 12 or SKILL_NODE)
        b:SetWidth(size)
        b:SetHeight(size)
        b:ClearAllPoints()
        b:SetPoint("CENTER", p.tree, "TOPLEFT", x, -y)
        b.node, b.skill = n, k
        b.icon:SetTexture(n.icon ~= "" and n.icon or "Interface\\Icons\\INV_Misc_QuestionMark")
        b.label:SetText(n.kind == 0 and "" or n.name)
        local rgb = SKILL_KIND_RGB[n.kind] or { 1, 0.82, 0 }
        local blocked = n.kind ~= 0 and skillBlocked(k, n)
        local lit = n.kind == 0 or n.rank > 0 or not blocked
        b.icon:SetDesaturated(not lit and 1 or nil)
        if n.sealed then
            b.border:SetTexture(0.64, 0.21, 0.93, 0.9)
            b.rank:SetText("Sealed")
            b.rank:SetTextColor(0.75, 0.45, 1)
        elseif n.kind == 0 then
            b.border:SetTexture(1, 0.82, 0, 1)
            b.rank:SetText("")
        else
            b.border:SetTexture(rgb[1], rgb[2], rgb[3], lit and 0.95 or 0.35)
            b.rank:SetText(n.rank .. "/" .. n.max)
            if n.rank >= n.max then
                b.rank:SetTextColor(1, 0.82, 0)
            elseif n.rank > 0 then
                b.rank:SetTextColor(0, 1, 0)
            else
                b.rank:SetTextColor(0.8, 0.8, 0.8)
            end
        end
        b:Show()
    end
    local li = 0
    for _, n in ipairs(k.nodes) do
        local from, to = pos[n.parent], pos[n.id]
        if from and to then
            li = li + 1
            p.lines[li] = p.lines[li] or {}
            local line = p.lines[li]
            local dx, dy = to[1] - from[1], to[2] - from[2]
            local count = math.max(1, math.ceil(math.sqrt(dx * dx + dy * dy) / 4))
            lineDots(line, p.tree, "BORDER", count)
            placeDots(line, p.tree, count, from[1], from[2], to[1], to[2], 4)
            if n.rank > 0 then
                colorDots(line, count, 1, 0.8, 0.3, 1)
            else
                colorDots(line, count, 0.35, 0.35, 0.4, 0.9)
            end
        end
    end
    for i, name in ipairs(k.branches) do
        local l = p.branches[i]
        if not l then
            l = p.tree:CreateFontString(nil, "OVERLAY", "GameFontNormal")
            p.branches[i] = l
        end
        local x, y = at(i - 1, 1)
        l:ClearAllPoints()
        l:SetPoint("BOTTOM", p.tree, "TOPLEFT", x, -(y - SKILL_NODE / 2 - 14))
        l:SetText(name)
        l:Show()
    end
end

function ArpgSkills_Redraw()
    skillsPaint()
end

-- The character sheet (ArpgSheet_Update, from the server's SMSG_ARPG_SHEET): what each attribute
-- does, what adds to each skill, and the ARPG totals, at the character's own numbers.
local sheetData = nil

-- Each "\n"-separated line of `text` into the open tooltip (the server colours them).
local function tipLines(text)
    if not text or text == "" then return end
    local at = 1
    while true do
        local cut = string.find(text, "\n", at, 1)
        local line = string.sub(text, at, cut and cut - 1 or string.len(text))
        if line ~= "" then GameTooltip:AddLine(line, 1, 1, 1, 1) end
        if not cut then break end
        at = cut + 1
    end
end

local STAT_NAMES = { "Strength", "Agility", "Stamina", "Intellect", "Spirit" }

-- A paper doll stat row's tooltip: the stock first line (the value and its buffs), then what the
-- attribute does here.
local function statTip()
    GameTooltip:SetOwner(this, "ANCHOR_RIGHT")
    GameTooltip:SetText(this.tooltip or STAT_NAMES[this.arpgStat] or "", 1, 1, 1)
    if sheetData and sheetData.attributes and sheetData.attributes[this.arpgStat] then
        tipLines(sheetData.attributes[this.arpgStat])
    else
        GameTooltip:AddLine("Waiting for the server's character sheet.", 0.6, 0.6, 0.6)
    end
    GameTooltip:Show()
end

local function hookStats()
    for i = 1, 5 do
        local frame = getglobal("CharacterStatFrame" .. i)
        if frame and not frame.arpgStat then
            frame.arpgStat = i
            frame:SetScript("OnEnter", statTip)
            frame:SetScript("OnLeave", function() GameTooltip:Hide() end)
        end
    end
end
hookStats()

function ArpgSheet_SummaryTip(button)
    GameTooltip:SetOwner(button, "ANCHOR_TOPLEFT")
    GameTooltip:SetText("Your ARPG totals", 1, 0.82, 0)
    if sheetData then tipLines(sheetData.summary) else GameTooltip:AddLine("Waiting for the server.", 0.6, 0.6, 0.6) end
    GameTooltip:Show()
end

function ArpgSkillBonus_Tip(button)
    local skill = skillSel and skillById(skillSel)
    GameTooltip:SetOwner(button, "ANCHOR_TOPLEFT")
    GameTooltip:SetText(skill and (skill.name .. ": what adds to it") or "Pick a skill", 1, 0.82, 0)
    local text = skill and sheetData and sheetData.skills and sheetData.skills[skill.id]
    if text then
        tipLines(text)
    elseif skill then
        GameTooltip:AddLine("Waiting for the server.", 0.6, 0.6, 0.6)
    end
    GameTooltip:Show()
end

function ArpgSheet_Update(sheet)
    sheetData = sheet
    hookStats()
end

function ArpgSkills_Update(skills)
    skillsData = skills
    if treeFrame and treeFrame:IsVisible() and treeFrame.tab == "skills" then skillsPaint() end
end

function ArpgTree_ShowTab(tab)
    local f = treeBuild()
    f.tab = tab
    for _, b in ipairs(f.tabs) do
        if b.tab == tab then b:Disable() else b:Enable() end
    end
    if tab == "skills" then
        f.title:SetText("Skills")
        f.webPane:Hide()
        skillsBuild(f):Show()
        skillsPaint()
    else
        f.title:SetText("Passive Web")
        if f.skillPane then f.skillPane:Hide() end
        f.webPane:Show()
        treeRedraw()
    end
end

function ArpgTree_Update(tree)
    treeData = tree
    if treeFrame and treeFrame:IsVisible() and treeFrame.tab ~= "skills" then treeRedraw() end
end

function ArpgTree_Toggle()
    if not treeData and not skillsData then
        treeAsk(3)
        DEFAULT_CHAT_FRAME:AddMessage("The passive web has not arrived from the server yet.")
        return
    end
    local f = treeBuild()
    if f:IsVisible() then
        f:Hide()
    else
        f:Show()
        ArpgTree_ShowTab(f.tab or (treeData and "web" or "skills"))
    end
end

-- Spells without ranks: the spellbook shows each spell without its "Rank N".
if SpellButton_UpdateButton then
    local stockUpdateButton = SpellButton_UpdateButton
    SpellButton_UpdateButton = function()
        stockUpdateButton()
        local sub = this and getglobal(this:GetName() .. "SubSpellName")
        local text = sub and sub:GetText()
        if text and string.find(text, "^Rank %d+$") then sub:SetText("") end
    end
end

-- The talent key and the talent micro button open the tree instead.
ToggleTalentFrame = ArpgTree_Toggle

-- A level gives a point: ask for the fresh count.
local treeEvents = CreateFrame("Frame")
treeEvents:RegisterEvent("PLAYER_LEVEL_UP")
treeEvents:SetScript("OnEvent", function() treeAsk(3) end)

-- ---------------------------------------------------------------------------------------------
-- The living world: the zone's event and the bounty board (cmangos Arpg/ArpgEvents.h), in a
-- tracker under the minimap. The client hands them over: ArpgEvent_Update(event or nil) twice a
-- second while one runs (with its clock and how far and which way its spot lies), and
-- ArpgBounties_Update(rows) on a change.

local EVENT_KIND = { "Purge", "Hunt", "Siege" }
local tracker = CreateFrame("Frame", "ArpgTracker", UIParent)
tracker:SetWidth(250)
tracker:SetHeight(40)
tracker:SetPoint("TOPRIGHT", UIParent, "TOPRIGHT", -18, -220)
tracker:SetFrameStrata("MEDIUM")

local function trackerText(parent, font, r, g, b)
    local s = parent:CreateFontString(nil, "OVERLAY", font)
    s:SetJustifyH("RIGHT")
    s:SetWidth(250)
    if r then s:SetTextColor(r, g, b) end
    return s
end

-- The event block: title, objective with progress, and the clock and the way there.
local eventBlock = CreateFrame("Frame", nil, tracker)
eventBlock:SetWidth(250)
eventBlock:SetHeight(52)
eventBlock:SetPoint("TOPRIGHT", tracker, "TOPRIGHT", 0, 0)
eventBlock.title = trackerText(eventBlock, "GameFontNormal", 1, 0.5, 0.05)
eventBlock.title:SetPoint("TOPRIGHT", eventBlock, "TOPRIGHT", 0, 0)
eventBlock.objective = trackerText(eventBlock, "GameFontHighlightSmall")
eventBlock.objective:SetPoint("TOPRIGHT", eventBlock.title, "BOTTOMRIGHT", 0, -2)
eventBlock.where = trackerText(eventBlock, "GameFontNormalSmall", 0.85, 0.85, 0.85)
eventBlock.where:SetPoint("TOPRIGHT", eventBlock.objective, "BOTTOMRIGHT", 0, -2)
eventBlock.bar = CreateFrame("StatusBar", nil, eventBlock)
eventBlock.bar:SetWidth(160)
eventBlock.bar:SetHeight(5)
eventBlock.bar:SetPoint("TOPRIGHT", eventBlock.where, "BOTTOMRIGHT", 0, -3)
eventBlock.bar:SetStatusBarTexture("Interface\\TargetingFrame\\UI-StatusBar")
eventBlock.bar:SetStatusBarColor(1, 0.5, 0.05)
eventBlock.bar:SetMinMaxValues(0, 1)
local eventBarBack = eventBlock.bar:CreateTexture(nil, "BACKGROUND")
eventBarBack:SetAllPoints(eventBlock.bar)
eventBarBack:SetTexture(0, 0, 0, 0.6)
eventBlock:Hide()

local bountyHeader = trackerText(tracker, "GameFontNormal", 1, 0.82, 0)
local bountyRows = {}
local bountyData = {}
local bountyConfirm = { slot = nil, at = 0 }

local function bountyRow(i)
    if bountyRows[i] then return bountyRows[i] end
    local row = CreateFrame("Button", "ArpgBountyRow" .. i, tracker)
    row:SetWidth(250)
    row:SetHeight(26)
    row.title = trackerText(row, "GameFontHighlightSmall")
    row.title:SetPoint("TOPRIGHT", row, "TOPRIGHT", 0, 0)
    row.progress = trackerText(row, "GameFontNormalSmall", 0.75, 0.75, 0.75)
    row.progress:SetPoint("TOPRIGHT", row.title, "BOTTOMRIGHT", 0, -1)
    row:RegisterForClicks("RightButtonUp")
    row:SetScript("OnEnter", function()
        local b = this.bounty
        if not b or not GameTooltip then return end
        GameTooltip:SetOwner(this, "ANCHOR_LEFT")
        GameTooltip:SetText(b.title, 1, 0.82, 0)
        GameTooltip:AddLine("Reward: " .. b.reward, 1, 1, 1, 1)
        GameTooltip:AddLine("Right-click twice to abandon it for another.", 0.6, 0.6, 0.6, 1)
        GameTooltip:Show()
    end)
    row:SetScript("OnLeave", function() if GameTooltip then GameTooltip:Hide() end end)
    row:SetScript("OnClick", function()
        local b = this.bounty
        if not b then return end
        local now = GetTime()
        if bountyConfirm.slot == b.slot and now - bountyConfirm.at < 4 then
            bountyConfirm.slot = nil
            treeAsk(13, b.slot)
        else
            bountyConfirm.slot = b.slot
            bountyConfirm.at = now
            DEFAULT_CHAT_FRAME:AddMessage("|cffffd200Right-click the bounty again to abandon it.|r")
        end
    end)
    bountyRows[i] = row
    return row
end

local function trackerLayout()
    local top = tracker
    local point = "TOPRIGHT"
    if eventBlock:IsShown() then
        top = eventBlock.bar
        point = "BOTTOMRIGHT"
    end
    bountyHeader:ClearAllPoints()
    bountyHeader:SetPoint("TOPRIGHT", top, point, 0, eventBlock:IsShown() and -10 or 0)
    local prev = bountyHeader
    local n = table.getn(bountyData)
    -- The bounties are the zone's own: its name heads them.
    local zone = GetZoneText and GetZoneText() or ""
    bountyHeader:SetText(n > 0 and (zone ~= "" and ("Bounties: " .. zone) or "Bounties") or "")
    for i = 1, 3 do
        local row = bountyRow(i)
        local b = bountyData[i]
        if b then
            row.bounty = b
            row.title:SetText(b.title)
            row.progress:SetText(b.have .. " / " .. b.need)
            row:ClearAllPoints()
            row:SetPoint("TOPRIGHT", prev, "BOTTOMRIGHT", 0, -3)
            row:Show()
            prev = row
        else
            row.bounty = nil
            row:Hide()
        end
    end
end

function ArpgEvent_Update(ev)
    if not ev then
        eventBlock:Hide()
        trackerLayout()
        return
    end
    local kind = EVENT_KIND[ev.kind] or "Event"
    eventBlock.title:SetText(ev.title)
    local need = ev.need > 0 and ev.need or 1
    if ev.state == 2 then
        eventBlock.title:SetTextColor(0.2, 1, 0.2)
        eventBlock.objective:SetText(kind .. " complete!")
        eventBlock.where:SetText("")
        eventBlock.bar:SetValue(1)
    elseif ev.state == 3 then
        eventBlock.title:SetTextColor(1, 0.25, 0.25)
        eventBlock.objective:SetText(kind .. " failed.")
        eventBlock.where:SetText("")
        eventBlock.bar:SetValue(ev.have / need)
    else
        eventBlock.title:SetTextColor(1, 0.5, 0.05)
        eventBlock.objective:SetText(ev.objective .. ": " .. ev.have .. " / " .. ev.need)
        local m = math.floor(ev.secs / 60)
        local s = ev.secs - m * 60
        local clock = string.format("%d:%02d", m, s)
        if ev.dir == "here" then
            eventBlock.where:SetText(clock .. "  |cff40ff40you are there|r")
        elseif ev.dir ~= "" then
            eventBlock.where:SetText(clock .. "  " .. ev.dist .. " yd " .. ev.dir)
        else
            eventBlock.where:SetText(clock)
        end
        eventBlock.bar:SetValue(ev.have / need)
    end
    eventBlock:Show()
    trackerLayout()
end

function ArpgBounties_Update(rows)
    bountyData = rows or {}
    trackerLayout()
end

-- ---------------------------------------------------------------------------------------------
-- The Forge (cmangos Arpg/ArpgCraft.h): put an item in (drag it onto the slot, or Alt-click it in
-- the bags or on the character while the Forge is open), then use an orb, a shard, or salvage it.
-- The purse comes as ArpgPurse_Update({orbs = {...}, shards = {...}}); a craft goes out through
-- the session-only setting arpgCraft, "op:bag:slot:affix:nonce" (bag 255 the character's slots).

BINDING_NAME_ARPGFORGE = "Forge"
local purse = { orbs = {}, shards = {} }
local forge, forgeItem = nil, nil
local craftNonce = 0
local lastPickup = nil
local SHARD_ROWS = 8
local shardPage = 1

local function craftAsk(op, affix)
    if not forgeItem then
        DEFAULT_CHAT_FRAME:AddMessage("|cffff4040Put an item in the Forge first.|r")
        return
    end
    craftNonce = craftNonce + 1
    if craftNonce >= 1000 then craftNonce = 1 end
    SetCVar("arpgCraft", op .. ":" .. forgeItem.bag .. ":" .. forgeItem.slot .. ":" .. (affix or 0) .. ":" .. craftNonce)
end

local function forgeItemTexture()
    if not forgeItem then return nil end
    if forgeItem.bag == 255 then
        return GetInventoryItemTexture("player", forgeItem.slot)
    end
    local texture = GetContainerItemInfo(forgeItem.bag, forgeItem.slot)
    return texture
end

local function forgeItemLink()
    if not forgeItem then return nil end
    if forgeItem.bag == 255 then
        return GetInventoryItemLink("player", forgeItem.slot)
    end
    return GetContainerItemLink(forgeItem.bag, forgeItem.slot)
end

local function forgePaint()
    if not forge then return end
    local texture = forgeItemTexture()
    if forgeItem and not texture then
        forgeItem = nil -- moved, sold or salvaged
    end
    if forgeItem then
        forge.icon:SetTexture(texture)
        forge.itemName:SetText(forgeItemLink() or "")
    else
        forge.icon:SetTexture("Interface\\PaperDoll\\UI-Backpack-EmptySlot")
        forge.itemName:SetText("|cff808080Drag an item here, or Alt-click one.|r")
    end
    for i = 1, 7 do
        local b = forge.orbs[i]
        local orb = purse.orbs[i]
        if orb then
            b.orb = orb
            b.count:SetText(orb.held)
            if orb.held > 0 then
                b.icon:SetVertexColor(1, 1, 1)
            else
                b.icon:SetVertexColor(0.35, 0.35, 0.35)
            end
            b:Show()
        else
            b:Hide()
        end
    end
    local n = table.getn(purse.shards)
    local pages = math.max(1, math.ceil(n / SHARD_ROWS))
    if shardPage > pages then shardPage = pages end
    for i = 1, SHARD_ROWS do
        local row = forge.shards[i]
        local s = purse.shards[(shardPage - 1) * SHARD_ROWS + i]
        if s then
            row.shard = s
            row.text:SetText(s.name .. " |cffffffffx" .. s.held .. "|r")
            row:Show()
        else
            row.shard = nil
            row:Hide()
        end
    end
    forge.page:SetText(n > 0 and (shardPage .. " / " .. pages) or "No shards yet: salvage items for them.")
    if shardPage > 1 then forge.prev:Enable() else forge.prev:Disable() end
    if shardPage < pages then forge.next:Enable() else forge.next:Disable() end
end

local function forgeSelect(bag, slot)
    forgeItem = { bag = bag, slot = slot }
    forgePaint()
end

-- The orb icons: vanilla's gems, as the orbs' items wear.
local ORB_ICON = {
    "Interface\\Icons\\INV_Misc_Gem_Pearl_04",
    "Interface\\Icons\\INV_Misc_Gem_Sapphire_02",
    "Interface\\Icons\\INV_Misc_Gem_Opal_01",
    "Interface\\Icons\\INV_Misc_Gem_Emerald_02",
    "Interface\\Icons\\INV_Misc_Gem_Crystal_01",
    "Interface\\Icons\\INV_Misc_Gem_Pearl_05",
    "Interface\\Icons\\INV_Misc_Gem_Diamond_02",
}

local function forgeBuild()
    if forge then return forge end
    local f = CreateFrame("Frame", "ArpgForgeFrame", UIParent)
    f:SetWidth(360)
    f:SetHeight(420)
    f:SetPoint("LEFT", UIParent, "LEFT", 80, 40)
    f:SetFrameStrata("DIALOG")
    f:EnableMouse(true)
    f:SetMovable(true)
    f:RegisterForDrag("LeftButton")
    f:SetScript("OnDragStart", function() this:StartMoving() end)
    f:SetScript("OnDragStop", function() this:StopMovingOrSizing() end)
    f:Hide()
    local bg = f:CreateTexture(nil, "BACKGROUND")
    bg:SetAllPoints(f)
    bg:SetTexture(0.03, 0.028, 0.04, 0.96)
    local edge = f:CreateTexture(nil, "BORDER")
    edge:SetPoint("TOPLEFT", f, "TOPLEFT", 0, -36)
    edge:SetPoint("TOPRIGHT", f, "TOPRIGHT", 0, -36)
    edge:SetHeight(1)
    edge:SetTexture(0.9, 0.8, 0.5, 0.35)
    local title = f:CreateFontString(nil, "OVERLAY", "GameFontNormalLarge")
    title:SetPoint("TOP", f, "TOP", 0, -10)
    title:SetText("Forge")
    local close = CreateFrame("Button", "ArpgForgeFrameClose", f, "UIPanelCloseButton")
    close:SetPoint("TOPRIGHT", f, "TOPRIGHT", -2, -2)

    -- The item slot.
    local slot = CreateFrame("Button", "ArpgForgeSlot", f)
    slot:SetWidth(40)
    slot:SetHeight(40)
    slot:SetPoint("TOPLEFT", f, "TOPLEFT", 16, -50)
    f.icon = slot:CreateTexture(nil, "ARTWORK")
    f.icon:SetAllPoints(slot)
    f.itemName = f:CreateFontString(nil, "OVERLAY", "GameFontHighlight")
    f.itemName:SetPoint("LEFT", slot, "RIGHT", 10, 0)
    f.itemName:SetWidth(280)
    f.itemName:SetJustifyH("LEFT")
    local function takeCursor()
        if CursorHasItem() and lastPickup then
            local bag, s = lastPickup.bag, lastPickup.slot
            ClearCursor()
            forgeSelect(bag, s)
        end
    end
    slot:RegisterForClicks("LeftButtonUp", "RightButtonUp")
    slot:SetScript("OnReceiveDrag", takeCursor)
    slot:SetScript("OnClick", function()
        if CursorHasItem() then
            takeCursor()
        elseif arg1 == "RightButton" then
            forgeItem = nil
            forgePaint()
        end
    end)
    slot:SetScript("OnEnter", function()
        if not forgeItem or not GameTooltip then return end
        GameTooltip:SetOwner(this, "ANCHOR_RIGHT")
        if forgeItem.bag == 255 then
            GameTooltip:SetInventoryItem("player", forgeItem.slot)
        else
            GameTooltip:SetBagItem(forgeItem.bag, forgeItem.slot)
        end
        GameTooltip:Show()
    end)
    slot:SetScript("OnLeave", function() if GameTooltip then GameTooltip:Hide() end end)

    -- The orbs.
    local orbsLabel = f:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    orbsLabel:SetPoint("TOPLEFT", f, "TOPLEFT", 16, -104)
    orbsLabel:SetText("Orbs")
    f.orbs = {}
    for i = 1, 7 do
        local b = CreateFrame("Button", "ArpgForgeOrb" .. i, f)
        b:SetWidth(36)
        b:SetHeight(36)
        b:SetPoint("TOPLEFT", f, "TOPLEFT", 16 + (i - 1) * 46, -124)
        b.icon = b:CreateTexture(nil, "ARTWORK")
        b.icon:SetAllPoints(b)
        b.icon:SetTexture(ORB_ICON[i])
        b.count = b:CreateFontString(nil, "OVERLAY", "NumberFontNormal")
        b.count:SetPoint("BOTTOMRIGHT", b, "BOTTOMRIGHT", -1, 2)
        b:SetScript("OnClick", function()
            if this.orb then craftAsk(this.orb.id) end
        end)
        b:SetScript("OnEnter", function()
            local o = this.orb
            if not o or not GameTooltip then return end
            GameTooltip:SetOwner(this, "ANCHOR_TOP")
            GameTooltip:SetText(o.name, 0.9, 0.8, 0.5)
            GameTooltip:AddLine(o.text, 1, 1, 1, 1)
            GameTooltip:AddLine("You hold " .. o.held .. ".", 0.7, 0.7, 0.7)
            GameTooltip:Show()
        end)
        b:SetScript("OnLeave", function() if GameTooltip then GameTooltip:Hide() end end)
        f.orbs[i] = b
    end

    -- The shards.
    local shardsLabel = f:CreateFontString(nil, "OVERLAY", "GameFontNormal")
    shardsLabel:SetPoint("TOPLEFT", f, "TOPLEFT", 16, -176)
    shardsLabel:SetText("Shards")
    local shardHint = f:CreateFontString(nil, "OVERLAY", "GameFontDisableSmall")
    shardHint:SetPoint("LEFT", shardsLabel, "RIGHT", 8, 0)
    shardHint:SetText("Click to add the affix, or raise it. Four per item.")
    f.shards = {}
    for i = 1, SHARD_ROWS do
        local row = CreateFrame("Button", "ArpgForgeShard" .. i, f)
        row:SetWidth(328)
        row:SetHeight(20)
        row:SetPoint("TOPLEFT", f, "TOPLEFT", 16, -194 - (i - 1) * 21)
        local hl = row:CreateTexture(nil, "HIGHLIGHT")
        hl:SetAllPoints(row)
        hl:SetTexture(0.9, 0.8, 0.5, 0.12)
        row.text = row:CreateFontString(nil, "OVERLAY", "GameFontNormalSmall")
        row.text:SetPoint("LEFT", row, "LEFT", 4, 0)
        row.text:SetTextColor(0.6, 0.8, 1)
        row:SetScript("OnClick", function()
            if this.shard then craftAsk(9, this.shard.affix) end
        end)
        f.shards[i] = row
    end
    f.prev = CreateFrame("Button", "ArpgForgePrev", f, "UIPanelButtonTemplate")
    f.prev:SetWidth(24)
    f.prev:SetHeight(20)
    f.prev:SetPoint("TOPLEFT", f, "TOPLEFT", 16, -366)
    f.prev:SetText("<")
    f.prev:SetScript("OnClick", function() shardPage = shardPage - 1 forgePaint() end)
    f.next = CreateFrame("Button", "ArpgForgeNext", f, "UIPanelButtonTemplate")
    f.next:SetWidth(24)
    f.next:SetHeight(20)
    f.next:SetPoint("LEFT", f.prev, "RIGHT", 4, 0)
    f.next:SetText(">")
    f.next:SetScript("OnClick", function() shardPage = shardPage + 1 forgePaint() end)
    f.page = f:CreateFontString(nil, "OVERLAY", "GameFontDisableSmall")
    f.page:SetPoint("LEFT", f.next, "RIGHT", 8, 0)

    -- Salvage: the item goes, for shards of its affixes and a chance at an orb.
    local salvage = CreateFrame("Button", "ArpgForgeSalvage", f, "UIPanelButtonTemplate")
    salvage:SetWidth(100)
    salvage:SetHeight(22)
    salvage:SetPoint("BOTTOMRIGHT", f, "BOTTOMRIGHT", -12, 10)
    salvage:SetText("Salvage")
    salvage.armedAt = 0
    salvage:SetScript("OnClick", function()
        if not forgeItem then
            craftAsk(8)
            return
        end
        if forgeItem.bag == 255 then
            DEFAULT_CHAT_FRAME:AddMessage("|cffff4040Take the item off to salvage it.|r")
            return
        end
        local now = GetTime()
        if now - this.armedAt < 4 then
            this.armedAt = 0
            this:SetText("Salvage")
            craftAsk(8)
            forgeItem = nil
            forgePaint()
        else
            this.armedAt = now
            this:SetText("Sure?")
        end
    end)
    salvage:SetScript("OnHide", function() this.armedAt = 0 this:SetText("Salvage") end)
    salvage:SetScript("OnEnter", function()
        if not GameTooltip then return end
        GameTooltip:SetOwner(this, "ANCHOR_TOP")
        GameTooltip:SetText("Salvage")
        GameTooltip:AddLine("Destroys the item for shards of its affixes (two from a blue or better) "
            .. "and a chance at an orb. Click twice.", 1, 1, 1, 1)
        GameTooltip:Show()
    end)
    salvage:SetScript("OnLeave", function() if GameTooltip then GameTooltip:Hide() end end)

    -- The bags change under the slot: repaint, so a moved item lets go.
    f:RegisterEvent("BAG_UPDATE")
    f:RegisterEvent("UNIT_INVENTORY_CHANGED")
    f:SetScript("OnEvent", function() if this:IsVisible() then forgePaint() end end)
    forge = f
    return f
end

function ArpgPurse_Update(p)
    purse = p or { orbs = {}, shards = {} }
    forgePaint()
end

function ArpgForge_Toggle()
    local f = forgeBuild()
    if f:IsVisible() then
        f:Hide()
    else
        f:Show()
        forgePaint()
    end
end

-- What was last picked up, so a drag onto the Forge's slot knows where it came from.
if PickupContainerItem then
    local stockPickupContainer = PickupContainerItem
    PickupContainerItem = function(bag, slot)
        lastPickup = { bag = bag, slot = slot }
        return stockPickupContainer(bag, slot)
    end
end
if PickupInventoryItem then
    local stockPickupInventory = PickupInventoryItem
    PickupInventoryItem = function(slot)
        lastPickup = { bag = 255, slot = slot }
        return stockPickupInventory(slot)
    end
end
-- Alt-click puts an item in the open Forge, from the bags or the character.
if ContainerFrameItemButton_OnClick then
    local stockContainerClick = ContainerFrameItemButton_OnClick
    ContainerFrameItemButton_OnClick = function(button, ignoreShift)
        if forge and forge:IsVisible() and IsAltKeyDown() and button == "LeftButton" then
            forgeSelect(this:GetParent():GetID(), this:GetID())
            return
        end
        return stockContainerClick(button, ignoreShift)
    end
end
if PaperDollItemSlotButton_OnClick then
    local stockPaperDollClick = PaperDollItemSlotButton_OnClick
    PaperDollItemSlotButton_OnClick = function(button, ignoreModifiers)
        if forge and forge:IsVisible() and IsAltKeyDown() and button == "LeftButton" then
            forgeSelect(255, this:GetID())
            return
        end
        return stockPaperDollClick(button, ignoreModifiers)
    end
end

-- The Forge's button, beside the town portal's.
local forgeButton = CreateFrame("Button", "ArpgHudForgeButton", UIParent)
forgeButton:SetWidth(28)
forgeButton:SetHeight(28)
forgeButton:SetPoint("LEFT", portalButton, "RIGHT", 6, 0)
forgeButton:SetFrameStrata("MEDIUM")
local forgeIcon = forgeButton:CreateTexture(nil, "ARTWORK")
forgeIcon:SetAllPoints(forgeButton)
forgeIcon:SetTexture("Interface\\Icons\\Trade_BlackSmithing")
forgeButton:SetScript("OnClick", function() ArpgForge_Toggle() end)
forgeButton:SetScript("OnEnter", function()
    if GameTooltip then
        GameTooltip:SetOwner(this, "ANCHOR_TOP")
        GameTooltip:SetText("Forge")
        GameTooltip:AddLine("Use orbs and shards on your items, or salvage them.", 1, 1, 1, 1)
        GameTooltip:Show()
    end
end)
forgeButton:SetScript("OnLeave", function() if GameTooltip then GameTooltip:Hide() end end)
