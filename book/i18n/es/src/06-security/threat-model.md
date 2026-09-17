# 06.2 — Modelo de amenazas

> **Status:** Especificado / Planificado  
> **Summary:** NUDO trata la salida de los modelos y la entrada de herramientas externas como no confiables hasta que reglas explícitas mueven los datos a través de las fronteras de confianza.

El modelo de amenazas incluye la inyección de prompts, la salida maliciosa de
herramientas, la escalada de capacidades, la exfiltración de secretos, la ejecución
sin límites, la pérdida de procedencia y los fallos de los adaptadores del runtime.

Las comprobaciones del lenguaje por sí solas no pueden proteger un sistema operativo.
Las reglas estáticas de efectos deben combinarse con la aplicación en tiempo de
ejecución y con el aislamiento allí donde los efectos secundarios cruzan fronteras de
proceso o de host.

Una implementación madura debería probar tanto las garantías positivas como las
negativas: no solo que los programas permitidos funcionan, sino que los efectos
denegados fallan *antes* de que el efecto ocurra.
