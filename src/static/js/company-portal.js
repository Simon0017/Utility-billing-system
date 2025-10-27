const API_BASE = '/api';

// Navigation
document.querySelectorAll('.nav-item').forEach(item => {
    item.addEventListener('click', (e) => {
        e.preventDefault();
        document.querySelectorAll('.nav-item').forEach(i => {
            i.classList.remove('text-white', 'bg-slate-700');
        });
        item.classList.add('text-white', 'bg-slate-700');
        
        const section = item.dataset.section;
        document.querySelectorAll('.dashboard-section').forEach(s => s.classList.add('hidden'));
        document.getElementById(section).classList.remove('hidden');
        
        if (section === 'dashboard') loadDashboard();
        else if (section === 'customers') loadCustomers();
        else if (section === 'meters') loadMeters();
        else if (section === 'readings') loadReadings();
        else if (section === 'payments') loadPayments();
        else if (section === 'reports') loadReports();
    });
});

// Modal handlers
document.querySelectorAll('.close-modal').forEach(btn => {
    btn.addEventListener('click', () => {
        btn.closest('.modal-overlay').classList.remove('show');
    });
});

document.getElementById('new-customer-btn').addEventListener('click', () => {
    document.getElementById('customer-modal').classList.add('show');
});

document.getElementById('add-customer-btn').addEventListener('click', () => {
    document.getElementById('customer-modal').classList.add('show');
});

document.getElementById('add-reading-btn').addEventListener('click', () => {
    document.getElementById('reading-modal').classList.add('show');
});

document.getElementById('add-reading-main-btn').addEventListener('click', () => {
    document.getElementById('reading-modal').classList.add('show');
});

document.getElementById('batch-meters-btn').addEventListener('click', () => {
    document.getElementById('batch-modal').classList.add('show');
});

document.getElementById('batch-gen-btn').addEventListener('click', () => {
    document.getElementById('batch-modal').classList.add('show');
});

// Form submissions
document.getElementById('customer-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const formData = new FormData(e.target);
    const data = Object.fromEntries(formData);
    
    try {
        const res = await fetch(`${API_BASE}/customers/`, {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify(data)
        });
        const result = await res.json();
        alert(`Customer registered! Password: ${result.password}`);
        e.target.reset();
        document.getElementById('customer-modal').classList.remove('show');
        loadCustomers();
    } catch (err) {
        alert('Error: ' + err.message);
    }
});

document.getElementById('reading-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const formData = new FormData(e.target);
    const data = Object.fromEntries(formData);

    if (data.reading) {
        data.reading = parseInt(data.reading,10)
    }
    
    try {
        await fetch(`${API_BASE}/add_readings/`, {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify(data)
        });
        alert('Reading added successfully!');
        e.target.reset();
        document.getElementById('reading-modal').classList.remove('show');
        loadReadings();
    } catch (err) {
        alert('Error: ' + err.message);
    }
});

document.getElementById('batch-form').addEventListener('submit', async (e) => {
    e.preventDefault();
    const formData = new FormData(e.target);
    const count = formData.get('count');
    
    try {
        const res = await fetch(`${API_BASE}/meters/batch/`, {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({count})
        });
        const result = await res.json();
        alert(`${result.meters.length} meters generated!`);
        e.target.reset();
        document.getElementById('batch-modal').classList.remove('show');
        loadMeters();
    } catch (err) {
        alert('Error: ' + err.message);
    }
});

document.getElementById('new-meter-btn').addEventListener('click', async () => {
    try {
        const res = await fetch(`${API_BASE}/meters/register/`, {method: 'POST'});
        const result = await res.json();
        alert(`Meter registered! Meter No: ${result.meter_no}`);
        loadMeters();
    } catch (err) {
        alert('Error: ' + err.message);
    }
});

document.getElementById('register-meter-btn').addEventListener('click', async () => {
    try {
        const res = await fetch(`${API_BASE}/meters/register/`, {method: 'POST'});
        const result = await res.json();
        alert(`Meter registered! Meter No: ${result.meter_no}`);
        loadMeters();
    } catch (err) {
        alert('Error: ' + err.message);
    }
});

// Load functions
async function loadDashboard() {
    try {
        const res = await fetch(`${API_BASE}/dashboard/stats/`);
        const data = await res.json();
        
        document.getElementById('stat-customers').textContent = data.total_customers;
        document.getElementById('stat-meters').textContent = data.active_meters;
        document.getElementById('stat-revenue').textContent = 'Ksh' + data.monthly_revenue.toLocaleString();
        document.getElementById('stat-pending').textContent = 'Ksh' + data.pending_payments.toLocaleString();
        
        renderRevenueChart(data.revenue_trend);
        renderPaymentStatusChart(data.payment_status);
    } catch (err) {
        console.error('Error loading dashboard:', err);
    }
}

async function loadCustomers() {    
    try {
        const res = await fetch(`${API_BASE}/customers/`);
        const customers = await res.json();
        console.log(customers);
        
        
        const tbody = document.getElementById('customers-table');
        tbody.innerHTML = customers.data.map(c => `
            <tr>
                <td class="px-6 py-4">${c.id}</td>
                <td class="px-6 py-4">${c.name}</td>
                <td class="px-6 py-4">${c.email}</td>
                <td class="px-6 py-4">${c.meter_no}</td>
                <td class="px-6 py-4 ${c.balance < 0 ? 'text-red-600' : ''}">$${c.balance}</td>
                <td class="px-6 py-4 text-right">
                    <button class="text-blue-600 hover:underline" onclick="viewCustomer(${c.id})">View</button>
                </td>
            </tr>
        `).join('');
    } catch (err) {
        console.error('Error loading customers:', err);
    }
}

async function loadMeters() {
    try {
        const res = await fetch(`${API_BASE}/meters/`);
        const meters = await res.json();
        
        const tbody = document.getElementById('meters-table');
        tbody.innerHTML = meters.meters.map(m => `
            <tr>
                <td class="px-6 py-4">${m.id}</td>
                <td class="px-6 py-4">${m.customer_name || 'Unassigned'}</td>
                <td class="px-6 py-4"><span class="status-badge ${m.status}">${m.status}</span></td>
                <td class="px-6 py-4">${m.last_reading || 'N/A'}</td>
                <td class="px-6 py-4 text-right">
                    <button class="text-blue-600 hover:underline" onclick="viewMeterHistory('${m.meter_no}')">History</button>
                </td>
            </tr>
        `).join('');
        
    } catch (err) {
        console.error('Error loading meters:', err);
    }
}

async function loadReadings() {
    try {
        const res = await fetch(`${API_BASE}/readings/`);
        const readings = await res.json();
        
        const tbody = document.getElementById('readings-table');
        tbody.innerHTML = readings.data.map(r => `
            <tr>
                <td class="px-6 py-4">${r.meter_no}</td>
                <td class="px-6 py-4">${r.reading}</td>
                <td class="px-6 py-4">${r.period}</td>
                <td class="px-6 py-4 ">${r.amount}</td>
                <td class="px-6 py-4">${new Date(r.date).toLocaleDateString()}</td>
                <td class="px-6 py-4 text-right">
                    <button class="text-blue-600 hover:underline readings-btn"  data-amount="${r.amount}"  data-meter_no = "${r.meter_no}">Invoice</button>
                </td>
            </tr>
        `).join('');

        document.querySelectorAll(".readings-btn").forEach(reading =>{
            const amount = reading.getAttribute("data-amount").trim();
            const meter_no = reading.getAttribute("data-meter_no").trim();
            reading.addEventListener("click",(e)=>{
                e.preventDefault();
                generateInvoice(meter_no,amount)
            })
        })
    } catch (err) {
        console.error('Error loading readings:', err);
    }
}

async function loadPayments() {
    try {
        const filter = document.getElementById('payment-filter').value;
        const res = await fetch(`${API_BASE}/payments/?filter=${filter}`);
        const payments = await res.json();
        
        const tbody = document.getElementById('payments-table');
        tbody.innerHTML = payments.map(p => `
            <tr>
                <td class="px-6 py-4">${p.customer_name}</td>
                <td class="px-6 py-4">${p.meter_no}</td>
                <td class="px-6 py-4">${p.amount}</td>
                <td class="px-6 py-4"><span class="status-badge ${p.status}">${p.status}</span></td>
                <td class="px-6 py-4">${new Date(p.date).toLocaleDateString()}</td>
                <td class="px-6 py-4 text-right">
                    <button class="text-blue-600 hover:underline" onclick="viewPayment(${p.id})">View</button>
                </td>
            </tr>
        `).join('');
    } catch (err) {
        console.error('Error loading payments:', err);
    }
}

document.getElementById('payment-filter').addEventListener('change', loadPayments);

async function loadReports() {
    try {
        const res = await fetch(`${API_BASE}/reports/analytics/`);
        const data = await res.json();
        
        renderConsumptionChart(data.consumption);
        renderDefaulterChart(data.defaulters);
    } catch (err) {
        console.error('Error loading reports:', err);
    }
}

// Chart functions
function renderRevenueChart(data) {
    const ctx = document.getElementById('revenue-chart').getContext('2d');
    new Chart(ctx, {
        type: 'line',
        data: {
            labels: data.labels,
            datasets: [{
                label: 'Revenue',
                data: data.values,
                borderColor: '#f59e0b',
                tension: 0.4
            }]
        },
        options: {responsive: true, maintainAspectRatio: false}
    });
}

function renderPaymentStatusChart(data) {
    const ctx = document.getElementById('payment-status-chart').getContext('2d');
    new Chart(ctx, {
        type: 'doughnut',
        data: {
            labels: ['Paid', 'Pending', 'Overdue'],
            datasets: [{
                data: [data.paid, data.pending, data.overdue],
                backgroundColor: ['#10b981', '#f59e0b', '#ef4444']
            }]
        },
        options: {responsive: true, maintainAspectRatio: false}
    });
}

function renderConsumptionChart(data) {
    const ctx = document.getElementById('consumption-chart').getContext('2d');
    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: data.labels,
            datasets: [{
                label: 'Consumption (kWh)',
                data: data.values,
                backgroundColor: '#f59e0b'
            }]
        },
        options: {responsive: true, maintainAspectRatio: false}
    });
}

function renderDefaulterChart(data) {
    const ctx = document.getElementById('defaulter-chart').getContext('2d');
    new Chart(ctx, {
        type: 'pie',
        data: {
            labels: data.labels,
            datasets: [{
                data: data.values,
                backgroundColor: ['#ef4444', '#f59e0b', '#10b981']
            }]
        },
        options: {responsive: true, maintainAspectRatio: false}
    });
}

// Utility functions
async function generateInvoice(readingId,amount) {
    try {
        const res = await fetch(`${API_BASE}/invoices/generate/${readingId}/`, {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({amount:amount})
        });
        const invoice = await res.json();
        alert('Invoice generated: #' + invoice.invoice_no);
    } catch (err) {
        alert('Error generating invoice: ' + err.message);
    }
}

async function viewMeterHistory(meterNo) {
    try {
        const res = await fetch(`${API_BASE}/meters/${meterNo}/history/`);
        const history = await res.json();
        console.log('Meter history:', history);
        // Display in modal or separate view
    } catch (err) {
        alert('Error loading history: ' + err.message);
    }
}

// Initialize
loadDashboard();