function format_date(date) {
    return new Date(Date.parse(date)).toLocaleTimeString()
}

function get_current_user() {
    const user_input = document.getElementById("user");
    return user_input.value;
}

function message_row(user, content, time, is_you) {
    const user_cell = document.createElement("td");
    user_cell.className = "user"
    user_cell.innerText = user;
    const content_cell = document.createElement("td");
    content_cell.className = "content"
    content_cell.innerText = content;
    const time_cell = document.createElement("td");
    time_cell.className = "time"
    time_cell.innerText = format_date(time);

    const row = document.createElement("tr");
    if (is_you) {
        row.className = "own_message"
    } else {
        row.className = "other_message"
    }
    row.appendChild(user_cell);
    row.appendChild(content_cell);
    row.appendChild(time_cell);

    return row;
}

function find_api_url() {
    const url = new URL(document.URL);
    url.port = "4000";
    url.pathname = "/";
    return url;
}

function initial_username() {
    const user = document.getElementById("user");
    if (user.value === "anon") {
        const num = "anon" + Math.floor(Math.random() * 90000) + 10000;
        user.value = num;
    }
}

const APIURL = find_api_url();

async function update_messages() {
    const response = await fetch(APIURL);

    if (!response.ok) {
        console.log(response)
    }

    const { time, messages } = await response.json();

    const time_div = document.getElementById("time");
    time_div.innerText = format_date(time);

    const current_user = get_current_user();

    const table = document.getElementById("messages");
    table.replaceChildren(...messages.map(({ user, content, time }) => {
        return message_row(user, content, time, user == current_user)
    }));
}

async function send_message() {
    const msg_input = document.getElementById("msg");
    const msg = msg_input.value;
    msg_input.value = "";

    const message = { user: get_current_user(), content: msg };

    const response = await fetch(APIURL, {
        mode: 'cors',
        method: 'post',
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify(message)
    });

    if (!response.ok) {
        console.log(response)
    }

    await update_messages()
}