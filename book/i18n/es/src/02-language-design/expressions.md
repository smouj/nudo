# 02.2 — Expresiones y precedencia

> **Status:** Especificado (SPECIFIED)  
> **Summary:** La precedencia, la asociatividad y la prohibición de la recursión por la izquierda están decididas y se aplican. El parser que las implementa es M2.

La precedencia no es una minucia de formato. `a + b * c` debe tener un único
significado en toda implementación conforme.

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
