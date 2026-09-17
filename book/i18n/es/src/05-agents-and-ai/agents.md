# 05.1 — Los agentes como ejecutores acotados

> **Status:** Propuesto  
> **Summary:** Un agente se concibe como un ejecutor declarado, con herramientas, autoridad, límites y criterios de aceptación explícitos, y no como un bucle de prompts sin restricciones.

Las declaraciones de agentes solo son útiles si comunican algo que el compilador o el
runtime puedan hacer cumplir. Por tanto, su propósito no es la organización
cosmética.

Quien revisa debería poder responder:

- ¿qué herramientas puede invocar este agente?
- ¿qué capacidades pueden consumir esas herramientas?
- ¿qué interfaz de modelo está disponible?
- ¿qué presupuesto limita la ejecución?
- ¿qué tipo de salida se espera?
- ¿qué verificación o aprobación se exige antes de continuar?

Los detalles de prompts y de peticiones propios de cada proveedor siguen siendo
responsabilidad del runtime.
