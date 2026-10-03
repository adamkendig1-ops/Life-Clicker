package main

import (
	"crypto/sha256"
	"encoding/base64"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"mime"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"
)

const launcherVersion = "2.1.1"
const productName = "Life Clicker"
const listenAddr = "127.0.0.1:8765"

type backupInfo struct {
	Name     string `json:"name"`
	Size     int64  `json:"size"`
	Modified string `json:"modified"`
}

type versionInfo struct {
	Version         string `json:"version"`
	SaveVersion     int    `json:"saveVersion"`
	LauncherVersion string `json:"launcherVersion"`
	Product         string `json:"product"`
}

type launcherConfig struct {
	Channel     string `json:"channel"`
	ManifestURL string `json:"manifestUrl"`
}

type onlineManifest struct {
	GameVersion            string   `json:"gameVersion"`
	SaveVersion            int      `json:"saveVersion"`
	MinimumLauncherVersion string   `json:"minimumLauncherVersion"`
	UpdateURL              string   `json:"updateUrl"`
	SHA256                 string   `json:"sha256"`
	Size                   int64    `json:"size"`
	Mandatory              bool     `json:"mandatory"`
	ReleaseNotes           []string `json:"releaseNotes"`
	LauncherVersion        string   `json:"launcherVersion"`
	LauncherURL            string   `json:"launcherUrl"`
	LauncherSHA256         string   `json:"launcherSha256"`
}

type updatePackage struct {
	Version           string   `json:"version"`
	SaveVersion       int      `json:"saveVersion"`
	AppHTML           string   `json:"appHtml"`
	Notes             []string `json:"notes"`
	LauncherExeBase64 string   `json:"launcherExeBase64,omitempty"`
}

type server struct {
	root           string
	httpServer     *http.Server
	shutdownOnce   sync.Once
	done           chan struct{}
	onlineMu       sync.Mutex
	onlineChecked  bool
	onlineResult   map[string]any
	onlineManifest *onlineManifest
}

func rootDir() string {
	exe, err := os.Executable()
	if err != nil {
		d, _ := os.Getwd()
		return d
	}
	return filepath.Dir(exe)
}
func stamp() string        { return time.Now().Format("20060102_150405_000") }
func openBrowser(u string) { _ = exec.Command("cmd", "/c", "start", "", u).Start() }

func setNoCache(w http.ResponseWriter) {
	w.Header().Set("Cache-Control", "no-store, no-cache, must-revalidate, max-age=0")
	w.Header().Set("Pragma", "no-cache")
	w.Header().Set("Expires", "0")
}
func writeJSON(w http.ResponseWriter, status int, v any) {
	setNoCache(w)
	w.Header().Set("Content-Type", "application/json; charset=utf-8")
	w.WriteHeader(status)
	_ = json.NewEncoder(w).Encode(v)
}
func prune(dir, prefix, suffix string, keep int) {
	entries, _ := os.ReadDir(dir)
	type f struct {
		name string
		mod  time.Time
	}
	var files []f
	for _, e := range entries {
		if e.IsDir() {
			continue
		}
		n := e.Name()
		if strings.HasPrefix(n, prefix) && strings.HasSuffix(n, suffix) {
			if info, err := e.Info(); err == nil {
				files = append(files, f{n, info.ModTime()})
			}
		}
	}
	sort.Slice(files, func(i, j int) bool { return files[i].mod.After(files[j].mod) })
	for i := keep; i < len(files); i++ {
		_ = os.Remove(filepath.Join(dir, files[i].name))
	}
}
func shaFile(path string) (string, int64, error) {
	f, err := os.Open(path)
	if err != nil {
		return "", 0, err
	}
	defer f.Close()
	h := sha256.New()
	n, err := io.Copy(h, f)
	if err != nil {
		return "", 0, err
	}
	return hex.EncodeToString(h.Sum(nil)), n, nil
}
func shaBytes(b []byte) string  { h := sha256.Sum256(b); return hex.EncodeToString(h[:]) }
func sameHash(a, b string) bool { return strings.EqualFold(strings.TrimSpace(a), strings.TrimSpace(b)) }

func handleStagedSelfUpdate(root string) bool {
	exe, err := os.Executable()
	if err != nil {
		return false
	}
	candidates := []string{filepath.Join(root, "LifeClickerLauncher.next.exe"), filepath.Join(root, "LifeClicker.next.exe")}
	var next string
	for _, p := range candidates {
		if st, err := os.Stat(p); err == nil && !st.IsDir() && st.Size() > 1024 {
			next = p
			break
		}
	}
	if next == "" {
		return false
	}
	helper := filepath.Join(os.TempDir(), "LifeClicker_selfupdate_"+stamp()+".cmd")
	prev := filepath.Join(root, "LifeClicker.previous.exe")
	script := fmt.Sprintf("@echo off\r\nping 127.0.0.1 -n 2 >nul\r\ndel /q %s 2>nul\r\nmove /y %s %s >nul\r\nmove /y %s %s >nul\r\nstart \"\" %s\r\ndel /q \"%%~f0\"\r\n", q(prev), q(exe), q(prev), q(next), q(exe), q(exe))
	if err := os.WriteFile(helper, []byte(script), 0644); err != nil {
		return false
	}
	_ = exec.Command("cmd", "/c", "start", "", "/min", helper).Start()
	return true
}
func q(s string) string { return `"` + strings.ReplaceAll(s, `"`, `\"`) + `"` }

func validRemoteURL(raw string) bool {
	u, err := url.Parse(strings.TrimSpace(raw))
	return err == nil && strings.EqualFold(u.Scheme, "https") && u.Host != ""
}
func cacheBust(raw string) string {
	u, err := url.Parse(raw)
	if err != nil {
		return raw
	}
	qv := u.Query()
	qv.Set("lc_nonce", strconv.FormatInt(time.Now().UnixNano(), 10))
	u.RawQuery = qv.Encode()
	return u.String()
}
func fetchRemote(raw string, limit int64, bust bool) ([]byte, error) {
	if !validRemoteURL(raw) {
		return nil, errors.New("remote URL must use HTTPS")
	}
	if bust {
		raw = cacheBust(raw)
	}
	req, err := http.NewRequest(http.MethodGet, raw, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Cache-Control", "no-cache, no-store, max-age=0")
	req.Header.Set("Pragma", "no-cache")
	req.Header.Set("User-Agent", "LifeClickerLauncher/"+launcherVersion)
	client := &http.Client{Timeout: 20 * time.Second, CheckRedirect: func(r *http.Request, via []*http.Request) error {
		if len(via) > 6 {
			return errors.New("too many redirects")
		}
		if !strings.EqualFold(r.URL.Scheme, "https") {
			return errors.New("redirect must remain HTTPS")
		}
		r.Header.Set("Cache-Control", "no-cache, no-store, max-age=0")
		r.Header.Set("Pragma", "no-cache")
		return nil
	}}
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		return nil, fmt.Errorf("HTTP %d", resp.StatusCode)
	}
	lr := io.LimitReader(resp.Body, limit+1)
	b, err := io.ReadAll(lr)
	if err != nil {
		return nil, err
	}
	if int64(len(b)) > limit {
		return nil, errors.New("remote file too large")
	}
	return b, nil
}
func versionGreater(a, b string) bool {
	parse := func(s string) []int {
		core := strings.SplitN(strings.TrimSpace(strings.TrimPrefix(s, "v")), "-", 2)[0]
		parts := strings.Split(core, ".")
		out := make([]int, 4)
		for i := 0; i < len(parts) && i < 4; i++ {
			n, _ := strconv.Atoi(parts[i])
			out[i] = n
		}
		return out
	}
	av, bv := parse(a), parse(b)
	for i := 0; i < len(av); i++ {
		if av[i] > bv[i] {
			return true
		}
		if av[i] < bv[i] {
			return false
		}
	}
	return false
}
func (s *server) currentVersion() versionInfo {
	v := versionInfo{Version: "0.0.0", SaveVersion: 0, LauncherVersion: launcherVersion, Product: productName}
	if b, err := os.ReadFile(filepath.Join(s.root, "game", "version.json")); err == nil {
		_ = json.Unmarshal(b, &v)
	}
	v.LauncherVersion = launcherVersion
	v.Product = productName
	if strings.TrimSpace(v.Version) == "" {
		v.Version = "0.0.0"
	}
	return v
}
func (s *server) readLauncherConfig() (launcherConfig, error) {
	var c launcherConfig
	b, err := os.ReadFile(filepath.Join(s.root, "launcher-config.json"))
	if err != nil {
		return c, err
	}
	err = json.Unmarshal(b, &c)
	if c.Channel == "" {
		c.Channel = "stable"
	}
	return c, err
}
func cloneMap(m map[string]any) map[string]any {
	n := map[string]any{}
	for k, v := range m {
		n[k] = v
	}
	return n
}
func (s *server) checkOnline(force bool) map[string]any {
	s.onlineMu.Lock()
	defer s.onlineMu.Unlock()
	if s.onlineChecked && !force && s.onlineResult != nil {
		return cloneMap(s.onlineResult)
	}
	cfg, err := s.readLauncherConfig()
	res := map[string]any{"configured": true, "checkedAt": time.Now().Format(time.RFC3339Nano)}
	if err != nil || strings.TrimSpace(cfg.ManifestURL) == "" {
		res["configured"] = false
		res["channel"] = cfg.Channel
		s.onlineChecked = true
		s.onlineResult = res
		return cloneMap(res)
	}
	res["channel"] = cfg.Channel
	res["manifestUrl"] = cfg.ManifestURL
	data, err := fetchRemote(cfg.ManifestURL, 2<<20, true)
	if err != nil {
		res["error"] = err.Error()
		s.onlineChecked = true
		s.onlineResult = res
		return cloneMap(res)
	}
	var m onlineManifest
	if err = json.Unmarshal(data, &m); err != nil {
		res["error"] = "invalid manifest: " + err.Error()
		s.onlineChecked = true
		s.onlineResult = res
		return cloneMap(res)
	}
	cur := s.currentVersion()
	gameAvail := m.GameVersion != "" && versionGreater(m.GameVersion, cur.Version)
	launchAvail := m.LauncherVersion != "" && versionGreater(m.LauncherVersion, launcherVersion)
	tooOld := m.MinimumLauncherVersion != "" && versionGreater(m.MinimumLauncherVersion, launcherVersion)
	res["available"] = gameAvail || launchAvail
	res["gameAvailable"] = gameAvail
	res["launcherAvailable"] = launchAvail
	res["launcherTooOld"] = tooOld
	res["currentGameVersion"] = cur.Version
	res["currentLauncherVersion"] = launcherVersion
	res["gameVersion"] = m.GameVersion
	res["saveVersion"] = m.SaveVersion
	res["launcherVersion"] = m.LauncherVersion
	res["mandatory"] = m.Mandatory
	res["releaseNotes"] = m.ReleaseNotes
	s.onlineManifest = &m
	s.onlineChecked = true
	s.onlineResult = res
	return cloneMap(res)
}

func copyFile(src, dst string) error {
	in, err := os.Open(src)
	if err != nil {
		return err
	}
	defer in.Close()
	if err = os.MkdirAll(filepath.Dir(dst), 0755); err != nil {
		return err
	}
	out, err := os.Create(dst)
	if err != nil {
		return err
	}
	_, cpErr := io.Copy(out, in)
	closeErr := out.Close()
	if cpErr != nil {
		return cpErr
	}
	return closeErr
}
func (s *server) applyUpdatePackage(u updatePackage) (map[string]any, error) {
	if strings.TrimSpace(u.Version) == "" || strings.TrimSpace(u.AppHTML) == "" {
		return nil, errors.New("invalid Life Clicker update package")
	}
	game := filepath.Join(s.root, "game", "index.html")
	if _, err := os.Stat(game); err != nil {
		return nil, errors.New("game/index.html was not found next to the launcher")
	}
	bdir := filepath.Join(s.root, "userdata", "game-backups")
	_ = os.MkdirAll(bdir, 0755)
	if cur, err := os.ReadFile(game); err == nil {
		_ = os.WriteFile(filepath.Join(bdir, "index_"+stamp()+".html"), cur, 0644)
	}
	prune(bdir, "index_", ".html", 12)
	tmp := game + ".new"
	if err := os.WriteFile(tmp, []byte(u.AppHTML), 0644); err != nil {
		return nil, err
	}
	if err := os.Rename(tmp, game); err != nil {
		_ = os.Remove(tmp)
		return nil, err
	}
	meta, _ := json.MarshalIndent(versionInfo{Version: u.Version, SaveVersion: u.SaveVersion, LauncherVersion: launcherVersion, Product: productName}, "", "  ")
	if err := os.WriteFile(filepath.Join(s.root, "game", "version.json"), meta, 0644); err != nil {
		return nil, err
	}
	staged := false
	if strings.TrimSpace(u.LauncherExeBase64) != "" {
		b, err := base64.StdEncoding.DecodeString(u.LauncherExeBase64)
		if err != nil {
			return nil, fmt.Errorf("launcher payload invalid: %w", err)
		}
		if len(b) < 1024 {
			return nil, errors.New("launcher payload too small")
		}
		next := filepath.Join(s.root, "LifeClickerLauncher.next.exe")
		if err = os.WriteFile(next, b, 0755); err != nil {
			return nil, fmt.Errorf("could not stage launcher update: %w", err)
		}
		staged = true
	}
	s.onlineChecked = false
	s.onlineResult = nil
	s.onlineManifest = nil
	return map[string]any{"ok": true, "version": u.Version, "gameInstalled": true, "launcherStaged": staged}, nil
}
func (s *server) installOnlineUpdate() (map[string]any, error) {
	status := s.checkOnline(true)
	if errText, _ := status["error"].(string); errText != "" {
		return nil, errors.New(errText)
	}
	avail, _ := status["available"].(bool)
	if !avail {
		return map[string]any{"ok": true, "gameInstalled": false, "launcherStaged": false, "message": "already up to date"}, nil
	}
	s.onlineMu.Lock()
	m := s.onlineManifest
	s.onlineMu.Unlock()
	if m == nil {
		return nil, errors.New("online manifest unavailable")
	}
	result := map[string]any{"ok": true, "gameInstalled": false, "launcherStaged": false}
	if ga, _ := status["gameAvailable"].(bool); ga {
		if !validRemoteURL(m.UpdateURL) {
			return nil, errors.New("update URL must use HTTPS")
		}
		b, err := fetchRemote(m.UpdateURL, 64<<20, true)
		if err != nil {
			return nil, err
		}
		if m.Size > 0 && int64(len(b)) != m.Size {
			return nil, fmt.Errorf("update size mismatch: got %d expected %d", len(b), m.Size)
		}
		if m.SHA256 != "" && !sameHash(shaBytes(b), m.SHA256) {
			return nil, errors.New("update SHA-256 verification failed")
		}
		var u updatePackage
		if err = json.Unmarshal(b, &u); err != nil {
			return nil, err
		}
		if u.Version != m.GameVersion {
			return nil, fmt.Errorf("package version %s does not match manifest %s", u.Version, m.GameVersion)
		}
		r, err := s.applyUpdatePackage(u)
		if err != nil {
			return nil, err
		}
		for k, v := range r {
			result[k] = v
		}
	}
	if la, _ := status["launcherAvailable"].(bool); la && validRemoteURL(m.LauncherURL) {
		b, err := fetchRemote(m.LauncherURL, 32<<20, true)
		if err != nil {
			return nil, err
		}
		if m.LauncherSHA256 != "" && !sameHash(shaBytes(b), m.LauncherSHA256) {
			return nil, errors.New("launcher SHA-256 verification failed")
		}
		if err = os.WriteFile(filepath.Join(s.root, "LifeClickerLauncher.next.exe"), b, 0755); err != nil {
			return nil, err
		}
		result["launcherStaged"] = true
	}
	return result, nil
}

func main() {
	root := rootDir()
	if handleStagedSelfUpdate(root) {
		return
	}
	baseURL := "http://" + listenAddr + "/launcher.html"
	ln, err := net.Listen("tcp", listenAddr)
	if err != nil {
		openBrowser(baseURL)
		return
	}
	s := &server{root: root, done: make(chan struct{})}
	mux := http.NewServeMux()
	srv := &http.Server{Handler: mux}
	s.httpServer = srv

	mux.HandleFunc("/api/version", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			http.Error(w, "GET required", 405)
			return
		}
		writeJSON(w, 200, s.currentVersion())
	})
	mux.HandleFunc("/api/backup-saves", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			http.Error(w, "POST required", 405)
			return
		}
		b, err := io.ReadAll(io.LimitReader(r.Body, 32<<20))
		if err != nil {
			writeJSON(w, 400, map[string]any{"error": err.Error()})
			return
		}
		var tmp any
		if json.Unmarshal(b, &tmp) != nil {
			writeJSON(w, 400, map[string]any{"error": "Invalid JSON"})
			return
		}
		dir := filepath.Join(root, "userdata", "backups")
		_ = os.MkdirAll(dir, 0755)
		name := "saves_" + stamp() + ".json"
		if err = os.WriteFile(filepath.Join(dir, name), b, 0644); err != nil {
			writeJSON(w, 500, map[string]any{"error": err.Error()})
			return
		}
		prune(dir, "saves_", ".json", 25)
		writeJSON(w, 200, map[string]any{"ok": true, "name": name})
	})
	mux.HandleFunc("/api/backups", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			http.Error(w, "GET required", 405)
			return
		}
		entries, _ := os.ReadDir(filepath.Join(root, "userdata", "backups"))
		var out []backupInfo
		for _, e := range entries {
			if e.IsDir() || !strings.HasPrefix(e.Name(), "saves_") || !strings.HasSuffix(e.Name(), ".json") {
				continue
			}
			if info, err := e.Info(); err == nil {
				out = append(out, backupInfo{Name: e.Name(), Size: info.Size(), Modified: info.ModTime().Format(time.RFC3339)})
			}
		}
		sort.Slice(out, func(i, j int) bool { return out[i].Name > out[j].Name })
		if out == nil {
			out = []backupInfo{}
		}
		writeJSON(w, 200, map[string]any{"backups": out})
	})
	mux.HandleFunc("/api/backup", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			http.Error(w, "GET required", 405)
			return
		}
		name := filepath.Base(r.URL.Query().Get("name"))
		if name == "" || name == "." {
			writeJSON(w, 400, map[string]any{"error": "Missing backup name"})
			return
		}
		b, err := os.ReadFile(filepath.Join(root, "userdata", "backups", name))
		if err != nil {
			writeJSON(w, 404, map[string]any{"error": "Backup not found"})
			return
		}
		setNoCache(w)
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		_, _ = w.Write(b)
	})
	mux.HandleFunc("/api/apply-update", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			http.Error(w, "POST required", 405)
			return
		}
		b, err := io.ReadAll(io.LimitReader(r.Body, 64<<20))
		if err != nil {
			writeJSON(w, 400, map[string]any{"error": err.Error()})
			return
		}
		var u updatePackage
		if json.Unmarshal(b, &u) != nil {
			writeJSON(w, 400, map[string]any{"error": "Invalid update package"})
			return
		}
		out, err := s.applyUpdatePackage(u)
		if err != nil {
			writeJSON(w, 500, map[string]any{"error": err.Error()})
			return
		}
		writeJSON(w, 200, out)
	})
	mux.HandleFunc("/api/online-update", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "GET" {
			http.Error(w, "GET required", 405)
			return
		}
		writeJSON(w, 200, s.checkOnline(r.URL.Query().Get("force") == "1"))
	})
	mux.HandleFunc("/api/install-online-update", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			http.Error(w, "POST required", 405)
			return
		}
		out, err := s.installOnlineUpdate()
		if err != nil {
			writeJSON(w, 500, map[string]any{"error": err.Error()})
			return
		}
		writeJSON(w, 200, out)
	})
	mux.HandleFunc("/api/integrity", func(w http.ResponseWriter, r *http.Request) {
		game := filepath.Join(root, "game", "index.html")
		launch := filepath.Join(root, "launcher.html")
		gsha, gb, gerr := shaFile(game)
		lsha, lb, lerr := shaFile(launch)
		writeJSON(w, 200, map[string]any{"ok": gerr == nil && lerr == nil, "game": map[string]any{"exists": gerr == nil, "bytes": gb, "sha256": gsha}, "launcher": map[string]any{"exists": lerr == nil, "bytes": lb, "sha256": lsha}})
	})
	mux.HandleFunc("/api/open-folder", func(w http.ResponseWriter, r *http.Request) {
		if r.Method != "POST" {
			http.Error(w, "POST required", 405)
			return
		}
		target := r.URL.Query().Get("target")
		p := root
		if target == "userdata" {
			p = filepath.Join(root, "userdata")
		}
		_ = os.MkdirAll(p, 0755)
		if err := exec.Command("explorer.exe", p).Start(); err != nil {
			writeJSON(w, 500, map[string]any{"error": err.Error()})
			return
		}
		writeJSON(w, 200, map[string]any{"ok": true})
	})
	mux.HandleFunc("/api/shutdown", func(w http.ResponseWriter, r *http.Request) {
		writeJSON(w, 200, map[string]any{"ok": true})
		s.shutdownOnce.Do(func() { go func() { time.Sleep(150 * time.Millisecond); _ = srv.Close(); close(s.done) }() })
	})
	mux.HandleFunc("/", func(w http.ResponseWriter, r *http.Request) {
		rel := strings.TrimPrefix(filepath.Clean("/"+r.URL.Path), "/")
		if rel == "" || rel == "." {
			rel = "launcher.html"
		}
		p := filepath.Join(root, filepath.FromSlash(rel))
		abs, err := filepath.Abs(p)
		if err != nil {
			http.Error(w, "Forbidden", 403)
			return
		}
		rabs, _ := filepath.Abs(root)
		if !strings.HasPrefix(strings.ToLower(abs), strings.ToLower(rabs+string(os.PathSeparator))) && abs != rabs {
			http.Error(w, "Forbidden", 403)
			return
		}
		info, err := os.Stat(abs)
		if err != nil || info.IsDir() {
			http.NotFound(w, r)
			return
		}
		if ct := mime.TypeByExtension(filepath.Ext(abs)); ct != "" {
			w.Header().Set("Content-Type", ct)
		}
		if strings.HasSuffix(strings.ToLower(rel), ".html") || strings.HasSuffix(strings.ToLower(rel), ".json") {
			setNoCache(w)
		}
		http.ServeFile(w, r, abs)
	})
	go func() { time.Sleep(300 * time.Millisecond); openBrowser(baseURL) }()
	_ = srv.Serve(ln)
	fmt.Print("")
}