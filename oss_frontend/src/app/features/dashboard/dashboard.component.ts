import {Component, OnInit} from '@angular/core';
import {Router, RouterLink, RouterOutlet} from '@angular/router';
import {AuthService} from '../../core/auth/auth.service';
import {AdminUser} from '../../shared/models/admin.model';
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
    private translate: TranslateService,
  ) {
    this.currentLang = this.translate.currentLang;
  }

  ngOnInit(): void {
    this.currentUser = this.authService.getCurrentUser();
    this.initializeLanguage();
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

  logout(): void {
    this.authService.logout();
    this.router.navigate(['/login']);
  }
}
