# Proyecto #1 Graficas

Proyecto de raycasting escrito en Rust para el curso de Graficas por
Computadora. El programa carga un laberinto desde `maze.txt`, renderiza un mapa
2D con rayos y permite cambiar a una proyeccion 3D simple en primera persona.

## Requisitos

- Toolchain de Rust con Cargo
- Un entorno de escritorio que pueda abrir una ventana de `minifb`

## Ejecutar

```bash
cargo run
```

## Controles

- `W`: avanzar
- `S`: retroceder
- `A`: girar a la izquierda
- `D`: girar a la derecha
- `Space`: saltar
- `Tab`: cambiar entre los modos de renderizado 2D y 3D
- `Esc`: cerrar la ventana

## Pruebas

```bash
cargo test
```

## Estructura del Proyecto

- `src/main.rs`: ciclo principal, configuracion de la ventana y cambio de modo
  de renderizado
- `src/maze.rs`: carga, validacion, consulta de celdas y reglas de movimiento
  del laberinto
- `src/player.rs`: posicion, angulo, campo de vision y estado de salto del
  jugador
- `src/input.rs`: manejo del teclado y validacion de colisiones al moverse
- `src/caster.rs`: logica de raycasting para rayos 2D e impactos de paredes 3D
- `src/render.rs`: renderizado del mapa 2D, proyeccion 3D y asignacion de
  colores
- `src/framebuffer.rs`: abstraccion del buffer de pixeles usado por el renderer
- `src/line.rs`: helper para dibujar lineas
- `maze.txt`: archivo editable con el laberinto

## Formato del Laberinto

El laberinto es un archivo de texto rectangular. Debe contener exactamente un
inicio del jugador (`p`) y una meta (`g`). El borde exterior debe estar formado
por paredes.

Caracteres reconocidos:

- `#`, `+`, `%`, `@`: paredes con diferentes colores
- espacio: piso transitable
- `p`: inicio del jugador
- `g`: meta
