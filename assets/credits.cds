content:
    - title: Septem Modi
      content: !RoleList
      - role: Game code
        name: InnocentusLime
      - role: Textures
        name: InnocentusLime and RiksM
    - title: Game engine
      content: !Subsections
      - title: Bevy engine
      - title: Third party plugins for bevy
        content: !RoleList
        - role: Bevy ecs tilemap
          name: StarArawn
        - role: Bevy mod debugdump
          name: jakobhellermann
        - role: iyes loopless
          name: inodentry (IyesGames)
        - role: Bevy asset loader
          name: NiklasEi
        - role: Bevy common asserts
          name: NiklasEi
        - role Bevy inspector egui
          name: jakobhellermann
      - title: Other dependencies
        content: !RoleList
        - role: serde
          name: dtolnay, serde-rs team
        - role: thiserror
          name: dtolnay
        - role: anyhow
          name: dtolnay
        - role: tiled
          name: mattyhall, bjorn and aleokdev
        - role: clap
          name: kbknapp, clap-rs team
        - role: image
          name: theotherphil, fintelia, HeroicKatora, image-rs team