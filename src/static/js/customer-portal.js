const API_BASE = '/api';
let currentMeterNo = '';
let currentBalance = 0;

// Search meter
document.getElementById('search-btn').addEventListener('click', searchMeter);
document.getElementById('meter-input').addEventListener('keypress', (e) => {
    if (e.key === 'Enter') searchMeter();
});

async function searchMeter() {
    const meterNo = document.getElementById('meter-input').value.trim();
    if (!meterNo) return alert('Please enter a meter number');
    
    document.getElementById('loading').classList.remove('hidden');
    document.getElementById('results').classList.add('hidden');
    
    try {
        const res = await fetch(`${API_BASE}/customer/meter/${meterNo}/`);
        if (!res.ok) throw new Error('Meter not found');
        
        const data = await res.json();
        currentMeterNo = meterNo;
        currentBalance = data.balance;
        
        displayResults(data);
    } catch (err) {
        alert('Error: ' + err.message);
    } finally {
        document.getElementById('loading').classList.add('hidden');
    }
}

function displayResults(data) {
    document.getElementById('results').classList.remove('hidden');
    
    // Account summary
    document.getElementById('balance').textContent = '$' + data.balance.toFixed(2);
    document.getElementById('last-payment').textContent = '$' + (data.last_payment || 0).toFixed(2);
    document.getElementById('latest-reading').textContent = data.latest_reading + ' kWh';
    
    // Latest invoice
    renderInvoice(data.latest_invoice);
    
    // History table
    const tbody = document.getElementById('history-table');
    tbody.innerHTML = data.history.map(h => `
        <tr>
            <td class="px-6 py-4">${h.period}</td>
            <td class="px-6 py-4">${h.reading} kWh</td>
            <td class="px-6 py-4">${h.consumption} kWh</td>
            <td class="px-6 py-4">$${h.amount}</td>
            <td class="px-6 py-4"><span class="status-badge ${h.status}">${h.status}</span></td>
        </tr>
    `).join('');
    
    // Charts
    renderConsumptionChart(data.consumption_trend);
    renderPaymentChart(data.payment_history);
}

function renderInvoice(invoice) {
    if (!invoice) {
        document.getElementById('invoice-content').innerHTML = '<p class="text-gray-500">No invoice available</p>';
        return;
    }
    
    document.getElementById('invoice-content').innerHTML = `
        <div class="invoice">
            <div class="flex justify-between mb-6">
                <div>
                    <h3 class="text-2xl font-bold text-amber-500">HEPHAESTUS MOTOR INC</h3>
                    <p class="text-sm text-gray-600">Energy Supply Services</p>
                </div>
                <div class="text-right">
                    <p class="font-semibold">INVOICE</p>
                    <p class="text-sm">#${invoice.invoice_no}</p>
                    <p class="text-sm">${new Date(invoice.date).toLocaleDateString()}</p>
                </div>
            </div>
            <div class="grid grid-cols-2 gap-4 mb-6">
                <div>
                    <p class="text-sm text-gray-600">Billed To:</p>
                    <p class="font-semibold">${invoice.customer_name}</p>
                    <p class="text-sm">Meter: ${invoice.meter_no}</p>
                </div>
                <div class="text-right">
                    <p class="text-sm text-gray-600">Billing Period:</p>
                    <p class="font-semibold">${invoice.period}</p>
                </div>
            </div>
            <table class="w-full mb-6">
                <thead class="bg-gray-50">
                    <tr>
                        <th class="px-4 py-2 text-left">Description</th>
                        <th class="px-4 py-2 text-right">Units</th>
                        <th class="px-4 py-2 text-right">Rate</th>
                        <th class="px-4 py-2 text-right">Amount</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td class="px-4 py-2">Energy Consumption</td>
                        <td class="px-4 py-2 text-right">${invoice.consumption} kWh</td>
                        <td class="px-4 py-2 text-right">$${invoice.rate}/kWh</td>
                        <td class="px-4 py-2 text-right">$${invoice.subtotal}</td>
                    </tr>
                    <tr>
                        <td class="px-4 py-2">Service Charge</td>
                        <td class="px-4 py-2 text-right">-</td>
                        <td class="px-4 py-2 text-right">-</td>
                        <td class="px-4 py-2 text-right">$${invoice.service_charge}</td>
                    </tr>
                </tbody>
                <tfoot class="border-t-2">
                    <tr>
                        <td colspan="3" class="px-4 py-2 text-right font-bold">Total Due:</td>
                        <td class="px-4 py-2 text-right font-bold text-red-600">$${invoice.total}</td>
                    </tr>
                </tfoot>
            </table>
            <p class="text-xs text-gray-500 text-center">Thank you for your business</p>
        </div>
    `;
}

// Invoice actions
document.getElementById('print-invoice').addEventListener('click', () => {
    window.print();
});

document.getElementById('download-invoice').addEventListener('click', () => {
    const element = document.getElementById('invoice-content');
    const opt = {
        margin: 1,
        filename: `invoice_${currentMeterNo}.pdf`,
        image: { type: 'jpeg', quality: 0.98 },
        html2canvas: { scale: 2 },
        jsPDF: { unit: 'in', format: 'letter', orientation: 'portrait' }
    };
    html2pdf().set(opt).from(element).save();
});

// Payment modal
document.getElementById('pay-now-btn').addEventListener('click', () => {
    document.getElementById('pay-meter-no').textContent = currentMeterNo;
    document.getElementById('pay-amount').textContent = '$' + currentBalance.toFixed(2);
    document.getElementById('confirm-amount').textContent = '$' + currentBalance.toFixed(2);
    document.getElementById('payment-modal').classList.add('show');
    nextStep(1);
});

document.querySelector('.close-modal').addEventListener('click', closePaymentModal);

function closePaymentModal() {
    document.getElementById('payment-modal').classList.remove('show');
}

function nextStep(step) {
    document.querySelectorAll('.payment-step').forEach(s => s.classList.add('hidden'));
    document.getElementById(`step-${step}`).classList.remove('hidden');
}

async function processPayment() {
    nextStep(4);
    
    try {
        await fetch(`${API_BASE}/customer/payment/`, {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({
                meter_no: currentMeterNo,
                amount: currentBalance
            })
        });
        
        setTimeout(() => {
            closePaymentModal();
            searchMeter(); // Refresh data
        }, 2000);
    } catch (err) {
        alert('Payment failed: ' + err.message);
    }
}

// Reports
document.querySelectorAll('.report-btn').forEach(btn => {
    btn.addEventListener('click', async () => {
        const type = btn.dataset.type;
        try {
            const res = await fetch(`${API_BASE}/customer/report/${currentMeterNo}/${type}/`);
            const blob = await res.blob();
            const url = window.URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `${type}_report_${currentMeterNo}.pdf`;
            document.body.appendChild(a);
            a.click();
            window.URL.revokeObjectURL(url);
            document.body.removeChild(a);
        } catch (err) {
            alert('Error downloading report: ' + err.message);
        }
    });
});

// Charts
function renderConsumptionChart(data) {
    const ctx = document.getElementById('consumption-trend-chart').getContext('2d');
    new Chart(ctx, {
        type: 'line',
        data: {
            labels: data.labels,
            datasets: [{
                label: 'Consumption (kWh)',
                data: data.values,
                borderColor: '#f59e0b',
                backgroundColor: 'rgba(245, 158, 11, 0.1)',
                tension: 0.4,
                fill: true
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: true,
            plugins: {
                legend: { display: false }
            }
        }
    });
}

function renderPaymentChart(data) {
    const ctx = document.getElementById('payment-history-chart').getContext('2d');
    new Chart(ctx, {
        type: 'bar',
        data: {
            labels: data.labels,
            datasets: [{
                label: 'Payments ($)',
                data: data.values,
                backgroundColor: '#10b981'
            }]
        },
        options: {
            responsive: true,
            maintainAspectRatio: true,
            plugins: {
                legend: { display: false }
            }
        }
    });
}