# 02.2 — Expresiones y precedencia

> **Status:** Implementado / Especificado / Planificado (IMPLEMENTED / SPECIFIED / PLANNED)
> **Summary:** El parser implementa la gramática; el comprobador también valida los tipos de operadores y las ramas de `if`. `match` y la indexación aún necesitan decisiones semánticas.

La precedencia no es una minucia de formato. `a + b * c` debe tener un único
significado en toda implementación conforme, y el parser produce exactamente esa
forma: en el árbol de `1 + 2 * 3` la multiplicación es el nodo interno y la suma está
por encima.

Estas ya no son cuestiones abiertas. Están enunciadas una sola vez, como una cadena
de niveles gramaticales en
[`grammar/nudo.ebnf`](https://github.com/smouj/nudo/blob/main/grammar/nudo.ebnf),
copiada idénticamente a los dos documentos que las describen, y comprobada
mecánicamente por
[`scripts/check-grammar.py`](https://github.com/smouj/nudo/blob/main/scripts/check-grammar.py):

- las operaciones postfijas se unen con más fuerza que los operadores unarios y
  encadenan, de modo que `a.b(c)[d]` es una sola expresión;
- la multiplicación y la división se unen con más fuerza que la suma y la resta,
  ambas asociativas por la izquierda;
- la comparación **no** encadena: `a < b < c` es un error de sintaxis y no
  `(a < b) < c`, y las comparaciones se combinan con `&&`;
- los operadores booleanos tienen una precedencia fija, con `||` como el más laxo;
- no hay nivel de asignación, porque todavía no está decidido si NUDO tiene enlaces
  mutables. Escribir ese nivel ahora sería inventar una característica del lenguaje
  para rellenar un hueco de un diagrama;
- `ask`, `verify` y `delegate` son expresiones primarias y no operadores, que es lo
  que mantiene analizable `verify draft with V`: `with` no es un operador y ninguna
  forma postfija comienza por él.

La regla que hace esto comprobable en lugar de aspiracional es la **ausencia de
recursión por la izquierda**. Un parser de descenso recursivo no puede tratar una
producción que derive una forma que empiece por sí misma, y la primera revisión de
esta gramática tenía cuatro producciones de ese tipo. Eso ahora es un fallo de
compilación, no un descubrimiento esperando a ocurrir.

El formateador debería imprimir suficiente sintaxis para preservar el significado
sin depender de la intuición humana.

## Reglas principales de operadores e `if`

[NEP-0017](https://github.com/smouj/nudo/blob/main/neps/0017-boolean-comparison-conditionals.md)
define los dominios de los operadores y los tipos resultantes de las condiciones:

- `!`, `&&` y `||` usan `Bool`; `&&` y `||` cortocircuitan de izquierda a
  derecha. Un operando no válido produce `NDO2018` en el operador.
- La igualdad compara vistas primitivas coincidentes de `Int`, `Float`, `Bool`
  o `Text`. El orden solo está definido para `Int` o `Float` coincidentes. No
  hay conversiones numéricas ni sobrecargas definidas por el usuario. La vista
  aceptada de `Verified<T>` como `T` también se aplica a los operadores.
- Las comparaciones de `Float` siguen IEEE-754: NaN no es igual a sí mismo y no
  tiene orden; el cero positivo y el negativo son iguales.
- La condición de `if` es `Bool`. Con `else`, un tipo esperado comprueba ambas
  ramas; sin él, ambas deben tener exactamente el mismo tipo. El desacuerdo
  produce `NDO2019` en la rama `else`.
- Un `if` sin `else` tiene tipo `Unit` y su rama `then` también debe ser `Unit`.

En ejecución, no se evalúan el operando omitido por cortocircuito ni la rama no
seleccionada. La comprobación estática de efectos considera todos ellos, para que
la declaración siga siendo válida con cualquier entrada.
