# PSA: Ballchasing boost stats are inaccurate

I’ve been digging into Ballchasing boost stats by comparing them against
replay-derived stats from `subtr-actor`, and the short version is:
Ballchasing’s boost numbers are not reliable, especially small pad counts.

This does not seem limited to one weird replay. Across nearly every replay I’ve
inspected, Ballchasing appears to undercount small boost pickups. The exact size
of the mismatch varies, but the pattern has been pretty consistent.

Here’s a concrete RLCS example: 2025 Worlds Grand Final, Team Falcons vs NRG,
Game 5.

Ballchasing page:
https://ballchasing.com/replay/b847c311-643d-4726-ab39-646a91a72b17

`subtr-actor` replay viewer, focused on BeastMode with small pad pickups
highlighted:
https://rlrml.github.io/subtr-actor/?replayUrl=https%3A%2F%2Fraw.githubusercontent.com%2Frlrml%2Fsubtr-actor%2Frlcs-worlds-grand-final-replay%2Fassets%2Frlcs-2025-worlds-grand-final-flcn-nrg-g5.replay#cfg=nVRNb9swDP0vOrtB_G3nNhTDWuxSNAV2GIpCselWiCwZkpwsC_Lf9yQn2YYmwNZLEJMi-fgeyT3bkLFCK7aIIzZIvlvxZs0We9aMxpByT6IntshmSRYxwx3-451di-FBW_dFc_lkuLLCIYVlC2dGmtxfRbPWXQdbx6WlQ8Qa3pPhPnWvW-RhnZZSb1nEOkP0YMiSYws1Shkx7hxv3qh9ACAy9y1eLx3xflEWeRHHdZVUWVqn2bxEeCus46qhZcMl8iazJI_Yikt5y_tjeVQfrdP9kpwT6hWo9ii_QTPxPGJvJF7fUDsuCnAgXPOG_7CfEqP_3HftRNcpsoiG027FhuRyIAI6eN2Zh5PtgKY16EUPoaADlVIo-rwBr7B8f0bU0fbI1St5G1tpEMvgAvstmc9dR83p9cTYiZa7EVWO7YWoB96eNZgMEGEcPinRcw9s8gGVoUabFkQEHgYEFejopP7jJDPeWTyR5LT6JlSrtx7FngmvxlHMENRQj4Z8qh9sUUVsF343graDNsG-Fa3znBYJCD6zHVdz1Ph5jzYRmALBRlix8hpOM3OIjtXOc_m-XlYXszou66IqoTpKxxkMSZ5WSRFncZqXGNz_xhJiLmP5zd215v2q_GvFa1WCfDdD0M9erZTnszTNqySd5_W8SubpB1pN_gARxuOAQcPcO_tO9GC9iQFnDRcM2LEbrw1OyFWMaZLM6iBIlidxVmUf0QOuv0FO03rxNNTVvCzLvKwByTs8m3IkfAGaEWHL9qGXEDzlWQTCZ7ZHSy8D9uilwa5h87DJh2dPCY7WKOlWq05M92PQ1uIYhL3as5UhvgZd6lZyb_cLC5jROj_6a7FlLDd8tMTvK76g_UJcDpvX36Xn7tBummxBg-ZvQ6H7gRthwbP2tAHUw88aJDW7P9Dp8kbd3gmR7x-VmcqAYWNDDoJVXKWKjWitvxNMTnXjoTzCQ9LhdJzUvgj0cfgE

For BeastMode in this replay, Ballchasing reports:

```text
small pads collected: 73
small boost collected: 769
total boost collected: 2532
```

`subtr-actor`, reading the replay directly, reports:

```text
small pads collected: 82
small boost collected: ~899.5
total boost collected: ~2667.2
```

The linked viewer is set up so you can watch BeastMode’s small pad pickup count
increment during playback. It reaches 82, not 73.

This same replay shows the same direction of mismatch for every player:

```text
Atomic      +10 small pads
Trk511      +10
BeastMode    +9
Daniel       +8
Rw9          +8
Kiileerrz    +5
```

So if you’re using Ballchasing boost stats for analysis, especially small pad
counts, I’d be careful. The numbers are good enough for a rough impression, but
they should not be treated as ground truth.

I’ve been building this out in the `subtr-actor` stats evaluation player, which
lets you inspect replay-derived stats alongside playback and visual overlays.
If you’re interested in replay analysis, boost routes, or validating stat
calculations visually, try playing around with the viewer link above.
