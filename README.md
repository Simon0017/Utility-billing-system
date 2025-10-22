# ⚙️ Hephaestus Utility Billing System

> **"Precision, Power, and Performance in Every Watt."**

A modern, high-performance **utility billing and monitoring platform** built in **Rust**, designed for speed, scalability, and real-time IoT integration.  
Powered by **Axum**, **Tera**, and **SeaORM**, the system provides smart metering, billing automation, and powerful RESTful APIs for enterprise use.

---

## 🏢 Company Information

**Virtual Company:** Hephaestus Motors Inc.  
**Department:** Embedded Systems & Smart Automation  
**Project Lead:** Simon Wekesa  
**Tech Stack:** Rust · Axum · SeaORM · Tera · PostgreSQL  

---

## 🚀 Overview

The **Hephaestus Utility Billing System** is a backend service for managing and visualizing power or water consumption data collected from IoT-enabled smart meters.

It is designed for:
- ⚡ Utility providers (electric, water, or gas)
- 🏢 Industrial environments
- 🧠 Research and automation systems

The API and web portal allow admins and clients to view real-time readings, generate usage reports, and process automated bills.

---

## 🔧 Core Features

| Feature | Description |
|----------|--------------|
| 💰 **Automated Billing Engine** | Compute billing from usage data with defined tariffs |
| 🧾 **Invoice Management** | Generate and store bills with timestamps and totals |
| 📊 **Usage Analytics** | Tera-powered dashboard for consumption visualization |
| 🧠 **Smart Alerts** | Detect anomalies or overuse events |
| 🔐 **Secure API** | Token-based access with middleware-based authentication |
| 🧩 **Modular Architecture** | Clean separation between routes, services, and models |

---

## 🧠 System Architecture


---

## 🦀 Tech Stack

| Layer | Technology |
|-------|-------------|
| **Web Framework** | [Axum](https://docs.rs/axum/latest/axum/) |
| **Template Engine** | [Tera](https://tera.netlify.app/docs/) |
| **ORM / Database** | [SeaORM](https://www.sea-ql.org/SeaORM/) + PostgreSQL |
| **Async Runtime** | Tokio |
| **Logging** | Tracing + EnvLogger |
| **Security** | Tower Middleware (Auth / Rate Limiting) |
| **Frontend (optional)** | Tera templates or REST API clients |

---

## 📂 Project Structure


---

## ⚙️ Setup and Installation

### 1️⃣ Prerequisites
- 🦀 Rust (latest stable)
- 🐘 PostgreSQL
- 🧱 SQLx CLI (optional, for migrations)
- Sea-ORM

### 2️⃣ Clone Repository
```bash
git https://github.com/Simon0017/Utility-billing-system.git
cd utility-billing-system
