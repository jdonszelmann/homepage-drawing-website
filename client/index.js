print = console.log

let socket;
let last;

let history = []
let online;
let MAXHISTORY = 1000
//const address = "ws://192.168.1.42:4242"
const address = "wss://drawserver.jonay2000.nl"


function checkhistory(){
    while(history.length > MAXHISTORY){
        history.shift();
    }
}

function setup(){
    print(`connecting to ${address}`);
    socket = new WebSocket(address)
    socket.onerror = () => {
        alert("could not make connection");
        location.reload();
    };


    online = 1;

    socket.onopen = function(){
        print("websocket connection made");
        socket.send(JSON.stringify(
            {
                command:"history"
            }
        ));
    }

    socket.onmessage = function(event){
        try{ 
            let data = JSON.parse(event.data); 

            if(data.command == "update"){
                if([data.x,data.y,data.oldx,data.oldy,data.color].some((x) => x < 0)){
                    print("dropping zero");
                    return;
                }
    
                
                online = data.numonline;
         
                if([data.x,data.y,data.oldx,data.oldy].some((x) => x < 0 || x > 100)){
                    print("dropping invalid input")
                    return;
                }
    
                colorMode(HSB,100);
                strokeWeight(5);
             
                stroke(data.color % 100, 50, 100)
                line(
                    map(data.x,0,100,0,windowWidth),
                    map(data.y,0,100,0,windowHeight),
                    map(data.oldx,0,100,0,windowWidth),
                    map(data.oldy,0,100,0,windowHeight), 
                )
                colorMode(RGB);


                history.push([data.x,data.y,data.oldx,data.oldy,data.color])
                redraw();
            }else if(data.command == "history"){
                online = data.numonline;
               
                history = data.history
                redraw();
            }
        }catch(e){
            print(`an error occured`, e);
            return;
        }
    }
    
    createCanvas(windowWidth,windowHeight);
    background(51);

    frameRate(0);
    last = createVector(-1,-1);

    noLoop();
}

function redrawHistory(){
    colorMode(HSB,100);
    strokeWeight(5);

    for(let i of history){    
        let [x,y,oldx,oldy,h] = i;
        stroke(h % 100, 50, 100)
        line(
            map(x,0,100,0,windowWidth),
            map(y,0,100,0,windowHeight),
            map(oldx,0,100,0,windowWidth),
            map(oldy,0,100,0,windowHeight), 
        )
    }
    colorMode(RGB);
}

function draw(){
    background(51);

    redrawHistory();

    fill(255);
    noStroke(255);
    textSize(14);
    text(`There are currently ${online} players online.`,20,20);
    
    checkhistory();
}

async function mouseReleased(){
    last = createVector(-1,-1);

    if (socket.readyState !== WebSocket.OPEN){
        return redraw();    
    } 

    await socket.send(JSON.stringify({
        command: "update",
        x: -1,
        y: -1
    }));

    redraw();
}

async function windowResized() {
    resizeCanvas(windowWidth, windowHeight);

    if (socket.readyState !== WebSocket.OPEN){
        return redraw();    
    }

    await socket.send(JSON.stringify({
        command: "update",
        x: -1,
        y: -1
    }));

    redraw();
}

async function mouseDragged() {

    if (socket.readyState !== WebSocket.OPEN){
        alert("still connecting");
        return
    }

    await socket.send(JSON.stringify({
        command: "update",
        x: map(mouseX,0,windowWidth,0,100),
        y: map(mouseY,0,windowHeight,0,100)
    }));
}
