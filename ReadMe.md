# Proyecto 1 - Raycasting

Proyecto universitario de Graficas por Computadora. El juego carga un laberinto
desde `maze.txt`, permite recorrerlo en vista 2D o 3D, y termina cuando el
jugador llega a la meta `g`.

## Tecnologia

- Rust
- minifb

## Funcionalidades

- Maze cargado desde archivo.
- Vista 2D de depuracion.
- Vista 3D estilo raycaster.
- Minimapa en la esquina durante la vista 3D.
- Raycasting con FOV.
- Movimiento con delta time.
- Contador de FPS visible durante gameplay.
- Colisiones y wall sliding.
- Fish-eye correction.
- Salto.
- Cinco colores normales de pared sin colores repetidos en paredes adyacentes.
- Paredes amarillas reservadas solo para la meta.
- Bresenham para rasterizacion de lineas, rayos y stakes.
- Meta y condicion de victoria.

## Ejecutar

```bash
cargo run
```

## Controles

Durante el juego:

```text
W       Avanzar
S       Retroceder
A       Girar izquierda
D       Girar derecha
Mouse   Girar camara horizontalmente
SPACE   Saltar
TAB     Alternar 2D / 3D
ESC     Salir
```

Despues de ganar:

```text
R       Reiniciar
ESC     Salir
```

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

- `#`, `+`, `%`, `@`, `&`: paredes con cinco colores diferentes
- `!`: paredes amarillas reservadas para la meta
- espacio: piso transitable
- `p`: inicio del jugador
- `g`: meta
