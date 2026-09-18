# 03.3 — Parser, sintaxis sin pérdidas y AST

> **Status:** Implementado (IMPLEMENTED)  
> **Summary:** M2 convirtió el flujo de tokens en una estructura sintáctica sin pérdidas, se recuperó de entradas malformadas y expuso un AST tipado para las fases semánticas posteriores.

## Por qué primero una sintaxis sin pérdidas

Un árbol sin pérdidas conserva los elementos accesorios y cada byte del código
fuente. Esa propiedad importa para un formateador, para las herramientas de edición y
para los agentes automatizados que necesitan modificar código sin destruir en
silencio comentarios o formato.

El parser la mantiene como una propiedad comprobada y no como una intención. Cada
fixture, cada caso de conformidad y cada prueba de propiedades verifica que
concatenar el texto de los tokens del árbol reproduce el archivo byte a byte. `nudo
check --dump-tree` imprime el árbol, y el volcado es más largo que el archivo del que
procede precisamente porque los espacios en blanco y los comentarios están dentro.

Ese coste honesto es la razón de que el árbol tenga la forma que tiene. `nudo-ast`
envuelve el árbol en valores tipados —una `Function` con un nombre y un cuerpo, un
`Binding` con un valor— y no devuelve nada cuando el árbol no contiene esa parte. Un
consumidor que necesita estar seguro, como un formateador que decide si puede
reescribir, tiene que poder distinguir «esto es una función» de «esto lo parecía
hasta el octavo token».

## Recuperación de errores

Un parser útil no se detiene en el primer token que falta. Los puntos de recuperación
incluyen los límites de elemento, los puntos y coma y los delimitadores de cierre, de
modo que un solo error no convierta el resto del archivo en ruido.

Tres reglas concretan eso, y cada una salió de un fallo que encontraron las pruebas:

* **La recuperación siempre consume un token o se detiene.** Una recuperación que
  entra en bucle es un cuelgue, y un cuelgue con la entrada del usuario es un bug.
* **La recuperación informa antes de saltarse algo.** Una versión temprana se saltaba
  en silencio lo que no sabía colocar, y `nudo check` salía entonces con `0` en un
  archivo que no había entendido. Eso es peor que rechazar el archivo, porque quien
  llama cree que se ha leído.
* **Una posición, un diagnóstico.** Si el lexer ya rechazó un byte, el parser no
  informa de la sentencia que ese byte volvió ilegible. Un carácter produce un
  mensaje, y un agente que repara el archivo arregla un byte.

Un token que el parser no sabe colocar se conserva en un nodo `Error` en lugar de
descartarse, de modo que el archivo todavía se puede reimprimir —incluidas las partes
que están mal.

## AST tipado

El AST ofrece a las fases posteriores formas semánticas (`Function`, `Call`,
`Type`) sin obligarlas a conocer los detalles crudos de los nodos sintácticos. Las
conversiones de la sintaxis al AST siguen preservando los spans, y el span de un nodo
comienza exactamente en su primer token.

Nada en `nudo-ast` resuelve un nombre ni comprueba un tipo: una ruta es una lista de
nombres, no una referencia. Eso es el hito M3, y el parser deliberadamente no sabe
nada de ello.

## Lo que falta deliberadamente

* Nada, por ahora: los parámetros genéricos en el punto de declaración
  (`enum Outcome<T, E>`) y el operando nombrado de `verify` se analizan desde M3.1,
  y cada uno tiene un caso de conformidad que lo fija. Tampoco hay ningún `return`
  que añadir: la EBNF nunca lo tuvo, y
  [NEP-0012](https://github.com/smouj/nudo/blob/main/neps/0012-early-exit.md)
  decide que la salida temprana es el valor de un bloque y no una sentencia. Un
  `return` suelto es un token inesperado y un `NDO1001`, fijado por
  `tests/conformance/parser/0017-no-return`.
* Las rutas de la gramática usan `::` y sus listas están separadas por comas, mientras
  que varios ejemplos del manual y de `examples/` escriben `web.search` uno por línea.
  El parser sigue la gramática; los ejemplos dicen que son avances, y qué grafía
  debería tener el lenguaje es un NEP, no un bug del parser.
