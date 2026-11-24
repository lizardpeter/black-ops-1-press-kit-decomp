#include "presskit/timeline.hpp"
#include "presskit/hit_test.hpp"
#include "presskit/background.hpp"
#include <QApplication>
#include <QWidget>
#include <QPainter>
#include <QMediaPlayer>
#include <QAudioOutput>
#include <QVideoSink>
#include <QVideoFrame>
#include <QDesktopServices>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QXmlStreamReader>
#include <QTimer>
#include <QElapsedTimer>
#include <QMouseEvent>
#include <QKeyEvent>
#include <QCache>
#include <QDebug>
#include <algorithm>
#include <cstdlib>
using namespace presskit;
static QString qs(std::string_view s){return QString::fromUtf8(s.data(),qsizetype(s.size()));}

class PressKit : public QWidget {
    QString root;
    Launcher launcher;
    Timeline timeline;
    Background background;
    QMediaPlayer player, music, fx;
    QAudioOutput audio, musicAudio, fxAudio;
    QVideoSink sink;
    QTimer timer;
    QElapsedTimer clock;
    QCache<int,QImage> frames{10};
    QImage shell, videoImage, screenshot;
    bool launched=false,smoke=false,videoSeen=false;
    double frameCredit=0, controlsY=1356;
    Hit hover{};
    QString error;
public:
    explicit PressKit(QString path,bool testing):root(QDir(path).absolutePath()),smoke(testing){
        setWindowTitle("Call of Duty: Black Ops — Press Kit");
        resize(1280,720);setMouseTracking(true);setFocusPolicy(Qt::StrongFocus);
        shell.load(root+"/art/launcher/1.png");
        player.setAudioOutput(&audio);player.setVideoSink(&sink);
        music.setAudioOutput(&musicAudio);fx.setAudioOutput(&fxAudio);
        musicAudio.setVolume(0.65f);fxAudio.setVolume(0.85f);
        if(smoke){audio.setMuted(true);musicAudio.setMuted(true);fxAudio.setMuted(true);}
        connect(&sink,&QVideoSink::videoFrameChanged,this,[this](const QVideoFrame& frame){
            if(frame.isValid()){videoImage=frame.toImage();videoSeen=!videoImage.isNull();update();}
        });
        connect(&player,&QMediaPlayer::mediaStatusChanged,this,[this](QMediaPlayer::MediaStatus s){
            if(s==QMediaPlayer::EndOfMedia){
                if(timeline.scene()==Scene::intro)timeline.intro_complete();
                else timeline.video_complete();
                dispatch();
            }
        });
        connect(&player,&QMediaPlayer::errorOccurred,this,[this](QMediaPlayer::Error, const QString& msg){error=msg;qWarning()<<msg;update();});
        connect(&timer,&QTimer::timeout,this,[this]{advance();});
        clock.start();timer.start(10);
    }
    bool assetsReady(){
        if(shell.isNull()||shell.size()!=QSize(500,300))return false;
        for(int i=1;i<=105;++i){QImage f(root+"/art/presskit/"+QString::number(i)+".png");if(f.isNull()||f.size()!=QSize(1280,720))return false;}
        for(auto p:{intro_path,*video_path(1),*video_path(2),*video_path(4),*video_path(5),*video_path(6)})
            if(!QFileInfo::exists(root+"/"+qs(p)))return false;
        return QFileInfo::exists(root+"/payload/screens/screens.xml");
    }
    void start(){if(launched)return;launcher.accept();launched=true;if(!smoke)showFullScreen();dispatch();update();}
    void smokeTest(){
        if(!assetsReady()){qCritical()<<"original assets missing or invalid";qApp->exit(2);return;}
        start();
        // Exercise a real original MP4 decode before switching to menu checks.
        auto* deadline=new QTimer(this);deadline->setSingleShot(true);
        connect(deadline,&QTimer::timeout,this,[]{qCritical()<<"No original intro video frame decoded";qApp->exit(3);});
        deadline->start(20000);
        auto* poll=new QTimer(this);
        connect(poll,&QTimer::timeout,this,[this,deadline,poll]{
            if(!videoSeen)return;deadline->stop();poll->stop();player.stop();
            timeline.intro_complete();
            for(int i=0;i<74;++i)timeline.tick();
            if(timeline.scene()!=Scene::welcome){qApp->exit(4);return;}
            timeline.navigate(WelcomeControl::wallpaper);for(int i=0;i<5;++i)timeline.tick();
            if(timeline.scene()!=Scene::wallpaper){qApp->exit(5);return;}
            timeline.navigate(WelcomeControl::screens);for(int i=0;i<5;++i)timeline.tick();
            if(!timeline.select_screenshot(20)){qApp->exit(6);return;}
            for(int i=0;i<4;++i)timeline.tick();dispatch();
            if(screenshot.isNull()||timeline.gallery.total!=20||timeline.gallery.index!=19){qApp->exit(7);return;}
            loadImage(timeline.gallery.previous_release());
            if(screenshot.isNull()||timeline.gallery.index!=18){qApp->exit(8);return;}
            timeline.close_media();for(int i=0;i<4;++i)timeline.tick();
            timeline.navigate(WelcomeControl::videos);for(int i=0;i<5;++i)timeline.tick();
            if(!timeline.select_video(4)){qApp->exit(9);return;}
            for(int i=0;i<4;++i)timeline.tick();videoSeen=false;dispatch();
            // Real trailer decode too, not only mocked host effect.
            deadline->start(20000);
            auto* p2=new QTimer(this);
            connect(p2,&QTimer::timeout,this,[this,p2,deadline]{
                if(!videoSeen)return;p2->stop();deadline->stop();player.stop();
                qInfo()<<"Original launcher, 105 frames, intro decode, menu transitions, 20-entry XML, gallery and trailer decode passed";
                qApp->exit(0);
            });p2->start(50);
        });poll->start(50);
    }
protected:
    void paintEvent(QPaintEvent*) override {
        QPainter p(this);p.fillRect(rect(),Qt::black);p.setRenderHint(QPainter::SmoothPixmapTransform);
        const double s=std::min(width()/1280.0,height()/720.0);
        p.translate((width()-1280*s)/2,(height()-720*s)/2);p.scale(s,s);
        if(!launched){p.drawImage(QRectF(390,210,500,300),shell);return;}
        auto scene=timeline.scene();
        if(scene==Scene::intro||scene==Scene::video_player){
            if(!videoImage.isNull())p.drawImage(QRectF(0,0,1280,720),videoImage);
        }else if(scene==Scene::gallery){
            if(!screenshot.isNull())p.drawImage(QRectF(0,0,1280,720),screenshot);
        }else {
            if(!frames.contains(timeline.frame)){
                auto* img=new QImage(root+"/art/presskit/"+QString::number(timeline.frame)+".png");
                frames.insert(timeline.frame,img);
            }
            if(auto* img=frames.object(timeline.frame))p.drawImage(QRectF(0,0,1280,720),*img);
        }
        if(scene==Scene::video_player||scene==Scene::gallery)drawControls(p,scene);
        if(!error.isEmpty()){p.setPen(Qt::white);p.drawText(QRectF(40,40,1200,80),Qt::TextWordWrap,error);}
    }
    void mouseMoveEvent(QMouseEvent* e) override{
        if(launched&&(timeline.scene()==Scene::video_player||timeline.scene()==Scene::gallery)){timeline.controls.mouse_activity();setCursor(Qt::ArrowCursor);}
        const auto h=hit(e->position());
        if(h.kind!=hover.kind||h.id!=hover.id){
            hover=h;if(launched&&h.kind!=HitKind::none&&h.kind!=HitKind::seek)playFx(28);
            if(h.kind!=HitKind::none)setCursor(Qt::PointingHandCursor);
            else if(!launched||timeline.scene()!=Scene::intro)setCursor(Qt::ArrowCursor);
        }
    }
    void mouseReleaseEvent(QMouseEvent* e) override{
        if(e->button()!=Qt::LeftButton)return;
        if(launched)timeline.controls.mouse_activity();
        const auto h=hit(e->position());
        switch(h.kind){
        case HitKind::accept:start();break;
        case HitKind::decline:launcher.decline();close();break;
        case HitKind::wallpaper_menu:timeline.navigate(WelcomeControl::wallpaper);playFx(2);break;
        case HitKind::videos_menu:timeline.navigate(WelcomeControl::videos);playFx(2);break;
        case HitKind::screens_menu:timeline.navigate(WelcomeControl::screens);playFx(2);break;
        case HitKind::website:timeline.navigate(WelcomeControl::website);break;
        case HitKind::quit:timeline.navigate(WelcomeControl::close);break;
        case HitKind::wallpaper:timeline.select_wallpaper(h.id);break;
        case HitKind::trailer:if(timeline.select_video(h.id))playFx(154);break;
        case HitKind::thumbnail:timeline.select_screenshot(h.id);break;
        case HitKind::close_media:timeline.close_media();break;
        case HitKind::previous:loadImage(timeline.gallery.previous_release());break;
        case HitKind::next:loadImage(timeline.gallery.next_release());break;
        case HitKind::play_pause:if(player.playbackState()==QMediaPlayer::PlayingState)player.pause();else player.play();break;
        case HitKind::seek:{const auto pt=stage(e->position());player.setPosition(qint64(player.duration()*std::clamp((pt.x()-241)/785,0.0,1.0)));break;}
        default:break;
        }
        dispatch();update();
    }
    void keyPressEvent(QKeyEvent* e) override{
        if(!launched){if(e->key()==Qt::Key_Return)start();else if(e->key()==Qt::Key_Escape)close();return;}
        if(e->key()==Qt::Key_Escape){
            if(!timeline.close_media()){if(timeline.scene()==Scene::intro){player.stop();timeline.intro_complete();}else close();}
        }else if(timeline.scene()==Scene::gallery){
            if(e->key()==Qt::Key_Left)loadImage(timeline.gallery.key_down(37));
            else if(e->key()==Qt::Key_Right)loadImage(timeline.gallery.key_down(39));
        }else if(timeline.scene()==Scene::video_player&&e->key()==Qt::Key_Space){
            if(player.playbackState()==QMediaPlayer::PlayingState)player.pause();else player.play();
        }
        dispatch();update();
    }
private:
    QPointF stage(QPointF p) const{
        double s=std::min(width()/1280.0,height()/720.0);
        if(s<=0)return {};
        return {(p.x()-(width()-1280*s)/2)/s,(p.y()-(height()-720*s)/2)/s};
    }
    Hit hit(QPointF p) const{
        auto pt=stage(p);
        if(!launched)return launcher_hit(pt.x()-390,pt.y()-210);
        if(timeline.scene()==Scene::gallery||timeline.scene()==Scene::video_player)
            return media_hit(timeline.scene(),pt.x(),pt.y(),controlsY);
        return menu_hit(timeline.scene(),pt.x(),pt.y());
    }
    void advance(){
        auto ms=clock.restart();if(!launched)return;
        frameCredit+=std::min<qint64>(ms,250);
        while(frameCredit>=1000.0/24){
            frameCredit-=1000.0/24;
            int old=timeline.frame;timeline.tick();
            if(old!=timeline.frame){
                if(timeline.frame==2)playFx(52);
                if(timeline.frame==56){background=Background{};startMusic();}
                if(timeline.frame==95||timeline.frame==105)controlsY=timeline.frame==95?1356:1346;
                dispatch();
            }
            if(timeline.frame>=56)background.advance(1000.0/24);
            if(timeline.scene()==Scene::gallery||timeline.scene()==Scene::video_player){
                timeline.controls.enter_frame(720);
                controlsY=slide_step(controlsY,timeline.controls.target_y);
                if(!timeline.controls.cursor.mouse_visible)setCursor(Qt::BlankCursor);
            }else setCursor(timeline.scene()==Scene::intro?Qt::BlankCursor:Qt::ArrowCursor);
        }
        update();
    }
    void playFx(int id){
        auto file=root+"/art/sounds/"+QString::number(id)+".mp3";
        if(QFileInfo::exists(file)){fx.stop();fx.setSource(QUrl::fromLocalFile(file));fx.play();}
    }
    void startMusic(){
        auto file=root+"/art/sounds/1.mp3";
        if(QFileInfo::exists(file)&&music.playbackState()!=QMediaPlayer::PlayingState){music.setLoops(QMediaPlayer::Infinite);music.setSource(QUrl::fromLocalFile(file));music.play();}
    }
    void play(std::string_view path){
        error.clear();videoImage=QImage{};videoSeen=false;
        player.setSource(QUrl::fromLocalFile(root+"/"+qs(path)));player.play();
    }
    void dispatch(){
        for(auto e:timeline.take_effects())switch(e.kind){
        case EffectKind::play_intro:case EffectKind::play_video:play(e.value);break;
        case EffectKind::stop_video:player.stop();videoImage=QImage{};break;
        case EffectKind::start_music:startMusic();break;
        case EffectKind::stop_music:music.stop();break;
        case EffectKind::load_screens_xml:loadXml();break;
        case EffectKind::open_url:
            if(!smoke){auto s=qs(e.value);QDesktopServices::openUrl(s.startsWith("http")?QUrl(s):QUrl::fromLocalFile(root+"/"+s));}break;
        case EffectKind::quit:close();break;
        }
    }
    void loadXml(){
        QFile f(root+"/payload/screens/screens.xml");
        std::vector<std::string> images;
        if(!f.open(QIODevice::ReadOnly)){timeline.gallery.xml_loaded(false,images);return;}
        QXmlStreamReader xml(&f);
        while(!xml.atEnd()){
            xml.readNext();
            if(xml.isStartElement()&&xml.name()==QStringLiteral("image")){
                auto path=xml.readElementText();
                if(!path.startsWith("payload/screens/")||path.contains("..")){error="Invalid image path";return;}
                images.push_back(path.toStdString());
            }
        }
        auto r=timeline.gallery.xml_loaded(!xml.hasError(),images);loadImage(r);
    }
    void loadImage(const std::optional<ImageRequest>& r){
        if(!r||r->index<0||r->index>=int(timeline.gallery.images.size()))return;
        auto file=root+"/"+QString::fromStdString(timeline.gallery.images[r->index]);
        QImage image(file);
        if(image.isNull()){error="Could not load "+file;return;}
        screenshot=std::move(image);
        timeline.gallery.loaded=timeline.gallery.filesize=QFileInfo(file).size();
    }
    void drawControls(QPainter& p,Scene scene){
        // Interim native chrome. Button locations and animation use original values,
        // but original button artwork must be layered in a later pass.
        p.save();p.translate(200,controlsY);
        p.fillRect(QRectF(0,-6,880,88),QColor(10,10,10,220));
        p.setPen(Qt::white);p.setFont(QFont("Sans Serif",14));
        p.drawText(QRectF(761,0,103,31),Qt::AlignCenter,"CLOSE");
        if(scene==Scene::gallery){
            p.drawText(QRectF(15,0,112,31),Qt::AlignCenter,"PREVIOUS");
            p.drawText(QRectF(165,0,115,31),Qt::AlignCenter,"NEXT");
            p.drawText(QRectF(300,0,440,31),Qt::AlignCenter,QString::fromStdString(picture_num(timeline.gallery.index,timeline.gallery.total)));
        }else{
            p.drawText(QRectF(15,0,112,31),Qt::AlignCenter,player.playbackState()==QMediaPlayer::PlayingState?"PAUSE":"PLAY");
            auto v=video_control_frame(controlsY,timeline.controls.target_y,player.position()/1000.0,player.duration()/1000.0);
            p.drawText(QRectF(300,0,440,31),Qt::AlignCenter,QString::fromStdString(v.display));
            p.fillRect(QRectF(41,56,785,5),QColor(90,90,90));
            if(player.duration()>0)p.fillRect(QRectF(41,56,785.0*player.position()/player.duration(),5),Qt::white);
        }
        p.restore();
    }
};

int main(int argc,char** argv){
    QApplication app(argc,argv);
    if(argc<2){qCritical()<<"Usage: presskit_player <prepared-original-assets-directory> [--smoke]";return 1;}
    const bool smoke=app.arguments().contains("--smoke");
    PressKit window(QString::fromLocal8Bit(argv[1]),smoke);window.show();
    if(smoke)QTimer::singleShot(0,&window,[&]{window.smokeTest();});
    return app.exec();
}
