import {Component, HostListener, OnInit} from '@angular/core';
import {Router, RouterLink, RouterOutlet} from '@angular/router';
import {AuthService} from '../../core/auth/auth.service';
import {AdminUser} from '../../shared/models/admin.model';
import {AdminService} from '../../shared/services/admin.service';
import {TranslateService} from '@ngx-translate/core';
import {NavbarComponent} from './components/navbar/navbar.component';

@Component({
  selector: 'app-dashboard',
  templateUrl: './dashboard.component.html',
  imports: [NavbarComponent, RouterLink, RouterOutlet],
  styleUrls: ['./dashboard.component.scss'],
})
export class DashboardComponent implements OnInit {
  currentUser: AdminUser | null = null;
  isUserMenuOpen = false;
  currentLang: string;

  constructor(
    private router: Router,
    private authService: AuthService,
    private adminService: AdminService,
    private translate: TranslateService,
  ) {
    this.currentLang = this.translate.currentLang;
  }

  ngOnInit(): void {
    this.loadCurrentUser();
    this.initializeLanguage();
  }

  private loadCurrentUser(): void {
    // First try to get from auth service
    const authUser = this.authService.getCurrentUser();
    if (authUser) {
      // Try to fetch full user details from API
      this.adminService.getMe().subscribe({
        next: (user: AdminUser) => {
          this.currentUser = user;
        },
        error: () => {
          // If API call fails, use basic user info
          this.currentUser = authUser as AdminUser;
        }
      });
    }
  }

  getUserDisplayName(): string {
    if (!this.currentUser) return 'User';
    if (this.currentUser.first_name && this.currentUser.last_name) {
      return `${this.currentUser.first_name} ${this.currentUser.last_name}`;
    }
    if (this.currentUser.first_name) {
      return this.currentUser.first_name;
    }
    return this.currentUser.email || 'User';
  }

  private initializeLanguage(): void {
    const savedLang = localStorage.getItem('preferred_language');
    if (savedLang) {
      this.setLanguage(savedLang);
    } else {
      const browserLang = navigator.language.split('-')[0];
      if (['en', 'es'].includes(browserLang)) {
        this.setLanguage(browserLang);
      } else {
        this.setLanguage('en');
      }
    }
  }

  switchLanguage(lang: string): void {
    this.setLanguage(lang);
    localStorage.setItem('preferred_language', lang);
  }

  private setLanguage(lang: string): void {
    this.translate.use(lang);
    this.currentLang = lang;
  }

  toggleUserMenu(): void {
    this.isUserMenuOpen = !this.isUserMenuOpen;
  }

  @HostListener('document:click', ['$event'])
  onDocumentClick(event: MouseEvent): void {
    const target = event.target as HTMLElement;
    if (!target.closest('.user-menu')) {
      this.isUserMenuOpen = false;
    }
  }

  logout(): void {
    this.authService.logout();
    this.router.navigate(['/login']);
  }
}
