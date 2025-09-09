import React from "react";
import ReactDOM from "react-dom/client";
import "./system-monitor"; // Esto inicia la lógica de monitoreo en segundo plano

const BackgroundApp = () => (
  <div style={{
    padding: '20px',
    fontFamily: 'Arial, sans-serif',
    textAlign: 'center'
  }}>
    <h2>System Monitor</h2>
    <p>Aplicación ejecutándose en segundo plano</p>
  </div>
);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BackgroundApp />
  </React.StrictMode>,
);
