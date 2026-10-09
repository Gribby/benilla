# ARPG skill trees: a new tree per class

The paladin tree is built (see "How it's built"). Jeff's call (`ARPG-ITEMISATION.md`, "Decisions so far"): ARPG players get a new
tree per class in place of vanilla's talents. This doc sets the framework every class shares, then
the paladin tree in full, since that's the class Jeff is testing. The other classes get a sketch
each, to be filled in the same way.

Read `ARPG-UNIQUES.md` first: the tree reuses its mechanic kit.

## What the tree is for

Vanilla's three talent tabs are tuned for group roles: a healer, a tank, a damage dealer. An ARPG
character plays alone and fights packs, so its tree should answer different questions:

- **How do I kill a pack?** Area and spread, not single-target throughput.
- **How do I stay alive alone?** Sustain and control in place of a healer and a tank.
- **What changes how a skill plays?** Diablo trees are memorable for nodes that change a skill's
  shape (Judgement now chains), not for +2% crit.

So the tree mixes three kinds of node, and the third kind is what makes it an ARPG tree.

## Node kinds

| Kind | What it does | Built from |
|---|---|---|
| **Passive** | A stat or a spell bonus, with ranks | Vanilla's own talent spells (Conviction's crit, Vengeance's damage). They are passive auras already, so a node just teaches rank N of that spell. |
| **Skill** | Teaches an active spell vanilla gates behind a talent | Vanilla's talent-granted spells (Consecration, Holy Shock, Seal of Command, Repentance, Holy Shield, the Blessings of Kings and Sanctuary) |
| **Modifier** | Changes how one skill works | The uniques' kit (chain, arc, spread, pierce, burst, echo…), as a tree source instead of an item |
| **Keystone** | A capstone that changes the class's whole loop | Small custom server code per keystone, one per branch |

**Modifiers and uniques stack sideways, not up.** The same kit on the same spell from a node and an
item doesn't double: the stronger one applies (the rule uniques already follow). Different kits
combine: Judgement that chains (node) and also bursts (Mograine's Might) is the build you're meant
to find.

## Points and pacing

Jeff's calls: the tree replaces talents, points are generous, respecs are free, and the three
paladin keystones stand.

- **One point per level past the first: 59 at 60.** The paladin tree holds 71 ranks, so a level 60
  fills two branches and part of a third. Points beyond vanilla's 51 are extra power. That's fine
  for a solo ARPG, and it's the first thing to trim if the tuning feels loose.
- **Three branches with three tiers and a keystone each.** A tier opens at 5 and 10 points spent in
  its branch, the keystone at 20. A talent rank or a taught spell also waits for the level vanilla
  gives that spell at (Holy Shock at 40), so a low-level character can't skip ahead of its spells.
- **Respec:** free, any time out of combat.

## The paladin tree

Three branches, each a different way to play a paladin solo. Level gates in brackets are when the
underlying vanilla spell first exists.

### Branch 1: Crusader (two-handed melee, from Retribution)

The swing-and-Judgement paladin. Packs die in front of you.

| Tier | Node | Kind | Ranks | Effect |
|---|---|---|---|---|
| 1 | Two-Handed Specialization | Passive | 3 | +2/4/6% damage with two-handed weapons (vanilla's talent) |
| 1 | Benediction | Passive | 5 | Judgement and Seals cost 3% less mana per rank (vanilla's) |
| 1 | Sweeping Seal | Modifier | 1 | Seal of Righteousness strikes every enemy in front of you; the extras take 40% (kit G) |
| 2 | Conviction | Passive | 5 | +1% crit per rank (vanilla's) |
| 2 | Chain of Judgement | Modifier | 2 | Judgement chains to 1/2 more enemies within 10 yd for 50% (kit C) |
| 2 | Seal of Command | Skill | 1 | Teaches Seal of Command (vanilla's talent spell) |
| 3 | Vengeance | Passive | 5 | +3% physical and Holy damage per rank for 8 sec after a crit (vanilla's) |
| 3 | Commanding Sweep | Modifier | 1 | Seal of Command's strikes hit every enemy in front of you at 60% (kit G) |
| Key | **Avenger** | Keystone | 1 | Judgement no longer consumes your Seal, but its cooldown doubles. You judge, keep swinging with the Seal up, and judge again. |

### Branch 2: Bulwark (shield and control, from Protection)

The paladin who doesn't need a healer: control the pack, punish anything that hits you.

| Tier | Node | Kind | Ranks | Effect |
|---|---|---|---|---|
| 1 | Toughness | Passive | 5 | +2% armour from items per rank (vanilla's) |
| 1 | Redoubt | Passive | 5 | Block chance after being crit (vanilla's) |
| 1 | Shockwave Hammer | Modifier | 2 | Hammer of Justice also stuns 1/2 more enemies within 8 yd (kit F) |
| 2 | Improved Retribution Aura | Passive | 2 | +25/50% Retribution Aura damage (vanilla's) |
| 2 | Reckoning | Passive | 5 | A chance after being crit to gain an extra attack (vanilla's) |
| 2 | Holy Shield | Skill | 1 | Teaches Holy Shield (vanilla's 31-point talent, here at 10 points) |
| 3 | Blessing of Sanctuary | Skill | 1 | Teaches it: blocked damage hurts the attacker (vanilla's) |
| 3 | Radiant Shield | Modifier | 1 | Holy Shield's holy damage bursts onto every enemy within 5 yd of the one it hits, at 50% (kit E) |
| Key | **Martyr's Ward** | Keystone | 1 | While you're in combat, Retribution Aura also strikes every enemy within 10 yd once a second, for double its damage. Standing in a pack is the plan. |

### Branch 3: Lightbringer (holy caster, from Holy)

The ranged paladin: holy damage at range, sustain from healing spells.

| Tier | Node | Kind | Ranks | Effect |
|---|---|---|---|---|
| 1 | Divine Intellect | Passive | 5 | +2% Intellect per rank (vanilla's) |
| 1 | Healing Light | Passive | 3 | +4/8/12% Holy Light and Flash of Light healing (vanilla's) |
| 1 | Consecration | Skill | 1 | Teaches Consecration (a talent in 1.12) |
| 2 | Illumination | Passive | 5 | Mana back on healing crits (vanilla's) |
| 2 | Holy Shock | Skill | 1 | Teaches Holy Shock (vanilla's 31-point talent, here at 10 points) |
| 2 | Arcing Shock | Modifier | 2 | Holy Shock chains to 1/2 more enemies within 10 yd for 60% (kit C) |
| 3 | Holy Power | Passive | 5 | +1% Holy spell crit per rank (vanilla's) |
| 3 | Purifying Light | Modifier | 1 | Exorcism and Holy Wrath strike every enemy, not only undead and demons |
| Key | **Dawnbringer** | Keystone | 1 | Your heals on yourself also send a bolt of holy damage at the nearest enemy, for 50% of the amount healed. Healing is your damage. |

### What a paladin build looks like at 60

- **Crusader 20 + Bulwark 20 + 11 spare:** a two-hander with Sweeping Seal and Chain of Judgement,
  Avenger, and Hammer of Justice stunning three. With Smite's Mighty Hammer early or Spinal
  Reaper late, Seal strikes cleave twice over.
- **Bulwark 20 + Lightbringer 20:** shield and Holy Shield, Martyr's Ward burning the pack while
  Holy Shock chains. Slow, very hard to kill.
- **Lightbringer 20 + Crusader:** Dawnbringer and Holy Shock at range, Judgement chaining in
  melee when something reaches you.

## How it's built

### Server (cmangos, `src/game/Arpg/ArpgTree.{h,cpp}`)

- **The tree is data in code**, as the uniques are: per node its branch, tier, column, kind, ranks,
  and what it gives. Talents are named, not numbered ("Conviction"): the server finds the class's
  talent by its rank 1 spell's name and teaches its rank N spell. Modifier and keystone spells are
  named too and resolved at first use; a name the data lacks logs an "ARPG tree:" error.
- **Saved per character** in `character_arpg_tree` (guid, node, rank), which the server creates
  if it's missing, so there's no SQL to run. Loaded at login, before the character's spells.
- **Applying a node:**
  - Passive: teaches the talent's rank N spell (dropping lower ranks).
  - Skill: teaches the talent's spell.
  - Modifier: a uniques-kit row the uniques' lookup also reads (`WornMechanic` is now "worn or
    learned", the larger count winning).
  - Keystone: a flag its hook checks. Avenger is in the Judgement script (`Paladin.cpp`), Martyr's
    Ward pulses from the ARPG swing update, Dawnbringer runs from the heal path, and Purifying
    Light from the creature-type checks in `Spell.cpp`.
- **Vanilla talents for an ARPG player:** a character with vanilla talents spent has them reset,
  free, at its first ARPG hello. The talent window learns nothing for an ARPG player. Spells the
  tree teaches are kept out of vanilla's talent-point accounting, so a level-up never resets them.
  A stock-client player keeps vanilla talents untouched.
- **Wire:** `CMSG_ARPG_ACTION` kind 8 spend (`u16` node), 9 respec, 10 query; `SMSG_ARPG_TREE`
  (0x33F) after the hello and every spend or respec: points, branch names, and per node its
  place, kind, ranks, icon spell, name and text. The client draws whatever the server defines, so
  a new node needs no client release.

### Client (benilla)

- `player/arpg/tree.rs` takes the server's tree and hands it to the ARPG HUD addon's window
  (`ArpgTree_Update` in `arpg_hud.lua`), with each node's icon path from the client's spell data.
- **The window:** the talent key (N) and the talent micro button open it in place of the talent
  frame. Three columns, one per branch, tiers as rows. A node shows its icon, its rank, a border in
  its kind's colour (keystones orange, modifiers the unique gold), and is greyed while locked. Hover
  for the tooltip; click to learn; Respec refunds everything.
- The window's requests go back through the session-only setting `arpgTreeAction`, a number
  (kind × 100000 + node × 100 + a nonce).

### Next

1. Play the paladin tree and tune it.
2. The other classes, one at a time, with the same framework (a table of nodes each).

## Other classes, in a line each

Each gets the same treatment: three solo-play branches, vanilla's talent spells as passives, kit
modifiers on its core skills, one keystone per branch.

| Class | Branch ideas | Keystone flavour |
|---|---|---|
| Mage | Pyromancer (Fireball fragments, Flamestrike), Frostbinder (Frostbolt pierce, Frost Nova shatter), Arcanist (Arcane Missiles chain, Blink) | Fire: every fire kill explodes |
| Warrior | Berserker (Whirlwind, Cleave arcs), Juggernaut (shield and Thunder Clap control), Warlord (shouts, Execute shockwave) | Rage never decays in a pack |
| Rogue | Duelist (Sinister Strike arcs), Shadow (step on kill, stealth openers), Poisoner (poisons spread on death) | Every kill resets Sprint |
| Hunter | Marksman (Arcane Shot fan, pierce), Beastmaster (pet cleave), Trapper (traps that chain) | Auto Shot fires two arrows |
| Priest | Inquisitor (Smite chains), Shadow (SW:P spreads, Mind Blast pierce), Penitent (heal-to-damage, like Dawnbringer) | Shadowform DoTs spread on kill |
| Warlock | Plaguebringer (DoT spread), Demonologist (pet arcs), Destroyer (Shadow Bolt pierce, Rain of Fire) | Every kill raises an imp |
| Shaman | Stormcaller (Lightning chains), Earthwarden (totems that pulse damage), Enhancer (Windfury arcs) | Lightning Bolt always chains |
| Druid | Moonkin (Wrath motes, Moonfire spread), Feral (Swipe-based pack cat or bear), Grovekeeper (Roots spread, Treants) | Shapeshifting is free |

## Open questions for Jeff

- A paragon-style track past 60 (more points for XP at max level) would give the tree a long
  tail. Worth doing once the endgame takes shape.
