print = console.log

let socket;
let history = []
let online;
let MAXHISTORY = 1024;
let context;
let canvas;
let drag = false;
let mouseX = 0;
let mouseY = 0;

const map = (value, x1, y1, x2, y2) => (value - x1) * (y2 - x2) / (y1 - x1) + x2;

// const address = "ws://localhost:4242"
const address = "wss://drawserver.jonay2000.nl"

const rtable = [255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 252.45, 244.79999999999998, 237.14999999999998, 229.49999999999997, 221.85, 214.2, 206.54999999999998, 198.9, 191.25, 183.6, 175.95, 168.29999999999998, 160.65000000000003, 153.00000000000003, 145.35000000000002, 137.70000000000002, 130.05, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 130.05000000000007, 137.70000000000002, 145.34999999999997, 152.99999999999991, 160.64999999999998, 168.30000000000004, 175.95, 183.59999999999994, 191.25, 198.90000000000006, 206.55, 214.19999999999996, 221.85000000000002, 229.50000000000009, 237.15000000000003, 244.79999999999998, 252.44999999999993, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0]
const gtable = [127.5, 135.15, 142.8, 150.45, 158.1, 165.75, 173.39999999999998, 181.04999999999998, 188.7, 196.35, 204.0, 211.65, 219.29999999999998, 226.95000000000002, 234.60000000000002, 242.25, 249.9, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 247.35, 239.7, 232.04999999999998, 224.39999999999998, 216.74999999999997, 209.09999999999997, 201.45000000000002, 193.80000000000007, 186.15, 178.50000000000006, 170.85, 163.20000000000005, 155.54999999999995, 147.9, 140.24999999999994, 132.6, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5]

const btable = [127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 127.5, 132.6, 140.24999999999994, 147.9, 155.54999999999995, 163.20000000000005, 170.85, 178.50000000000006, 186.15, 193.8, 201.45000000000002, 209.10000000000002, 216.75000000000003, 224.40000000000003, 232.04999999999998,
239.7, 247.35, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 255.0, 249.9, 242.25000000000006, 234.6, 226.95000000000005, 219.29999999999995, 211.65, 203.99999999999994, 196.35, 188.69999999999993, 181.04999999999998, 173.40000000000003, 165.75000000000009, 158.10000000000002, 150.44999999999996, 142.8, 135.15000000000006]

//gets color from internal color value between 0 and 100
function getColor(i){
    return `color: rgb(${rtable[i]},${gtable[i]},${btable[i]});`
}

function checkhistory(){
    while(history.length > MAXHISTORY){
        history.shift();
    }
}

window.onload = function(){

    canvas = document.createElement('canvas');
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;
    canvas.style.zIndex = 8;
    canvas.style.position = "absolute";
    context = canvas.getContext("2d");
    document.getElementsByTagName("body")[0].appendChild(canvas);
    context.lineCap = "round"

    print(`connecting to ${address}`);
    socket = new WebSocket(address)
    socket.onerror = () => {
        alert("could not make connection");
        location.reload();
    };
    socket.onclose = (e) => {
        if(e.code == 1008){
            alert("You have been blacklisted from this website.")
        }else{
            alert("lost connection connection");
            location.reload();
        }
    };

    online = 1;

    socket.onopen = function(event){
        print("websocket connection made");
    }

    socket.onmessage = function(event){
        try{
            let data = JSON.parse(event.data);

            if(data.command == "update"){
                if([data.x,data.y,data.oldx,data.oldy,data.color].some((x) => x < 0)){
                    //print("dropping zero");
                    return;
                }


                online = data.numonline;

                if([data.x,data.y,data.oldx,data.oldy].some((x) => x < 0 || x > 100)){
                    //print("dropping invalid input")
                    return;
                }


                context.strokeStyle = `rgb(${rtable[data.color]},${gtable[data.color]},${btable[data.color]})`
                context.lineWidth = 5;
                context.beginPath();
                context.moveTo(
                    map(data.x,0,100,0,canvas.width),
                    map(data.y,0,100,0,canvas.height),
                );
                context.lineTo(
                    map(data.oldx,0,100,0,canvas.width),
                    map(data.oldy,0,100,0,canvas.height),
                )
                context.stroke();

                history.push([data.x,data.y,data.oldx,data.oldy,data.color])
                update_screen();
            }else if(data.command == "history"){
                online = data.numonline;
                MAXHISTORY = data.capacity;
                history = data.history
                console.info("%c your color is: %s", getColor(data.color), data.color);
                update_screen();
            }
        }catch(e){
            print(`an error occured`, e);
            return;
        }
    }
}

function drawHistory(){
    context.lineWidth = 5;
    for(const i of history){  
        context.strokeStyle =`rgb(${rtable[i[4]]},${gtable[i[4]]},${btable[i[4]]})`
        context.beginPath();
        context.moveTo(
            map(i[0],0,100,0,canvas.width),
            map(i[1],0,100,0,canvas.height),
        );
        context.lineTo(
            map(i[2],0,100,0,canvas.width),
            map(i[3],0,100,0,canvas.height), 
        );
        context.stroke();
   } 
}

function update_screen(){
    window.requestAnimationFrame(() => {
        context.fillStyle = "rgb(51,51,51)";
        context.fillRect(0, 0, canvas.width,canvas.height);

        drawHistory();

        context.fillStyle = "white";
        context.font = "14px Arial";
        context.fillText(`There are currently ${online} players online.`, 20, 20);
    
        checkhistory();
    })
}

window.addEventListener('resize', () => {
    canvas.width = window.innerWidth;
    canvas.height = window.innerHeight;

    if (socket.readyState !== WebSocket.OPEN){
        return update_screen();    
    }

    socket.send(JSON.stringify({
        x: -1,
        y: -1
    }));

    update_screen();

}, false)

document.addEventListener('mouseup',onrelease, false);
document.addEventListener('mousedown', () => drag = true, false);
document.addEventListener('mousemove', onmousemove, false);
window.addEventListener('touchmove', ontouchmove, false);
window.addEventListener('touchend', onrelease, false);

function getTouchPos(canvasDom, touchEvent) {
    var rect = canvasDom.getBoundingClientRect();
    return [
        touchEvent.touches[0].clientX - rect.left,
        touchEvent.touches[0].clientY - rect.top
    ];
}

function ontouchmove(e){
    [mouseX,mouseY] = getTouchPos(canvas,e);
    handlemove();
}

function onrelease(){
    if (socket.readyState !== WebSocket.OPEN){
        return update_screen();    
    } 

    socket.send(JSON.stringify({
        x: -1,
        y: -1
    }));

     update_screen();

    drag = false
}

function handlemove(){
    if (socket.readyState !== WebSocket.OPEN){
        ////alert("still connecting\n please wait or reload the page.");
        return
    }
    
    socket.send(JSON.stringify({
        x: map(mouseX,-1,canvas.width,0,100),
        y: map(mouseY,-1,canvas.height,0,100)
    }));
}

function onmousemove(e){
    if(drag){
        mouseX = e.pageX;
        mouseY = e.pageY;
        handlemove(e);
    }
}
